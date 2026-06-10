# Copyright (C) 2026
# Wassim Jabi <wassim.jabi@gmail.com>
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later
# version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
# details.
#
# You should have received a copy of the GNU Affero General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.
"""
Building energy modelling for topologic-fast.

This is a port of ``topologicpy.EnergyModel`` to the topologic-fast API. It
consumes topologic-fast ``Cell`` / ``CellComplex`` buildings, extracts their
faces with topologic-fast geometry, and drives OpenStudio with exactly the same
model-construction logic as topologicpy (same OSM template, weather/design-day
files, surface classification, thermal-zone/thermostat setup). For identical
input geometry the resulting OpenStudio model is equivalent to the one
topologicpy produces.

Requirements / differences from topologicpy (documented for parity):
  * Requires the ``openstudio`` Python bindings. They currently only provide
    working wheels for CPython <= 3.12 (they import but crash on construction
    under 3.13/3.14), so these features run on a 3.12 interpreter.
  * The default OSM template / EPW / DDY assets are taken from the installed
    ``topologicpy`` package so both libraries build from identical inputs.
  * topologic-fast does not expose per-face apertures or dictionaries at the
    Python layer, so window placement uses the ``glazingRatio`` path
    (``setWindowToWallRatio``) rather than explicit aperture sub-surfaces.
"""
from __future__ import annotations

import math
import os
import warnings


def _openstudio():
    """Imports and returns the openstudio module (quietly), or None."""
    try:
        import openstudio
        openstudio.Logger.instance().standardOutLogger().setLogLevel(openstudio.Fatal)
        return openstudio
    except Exception:
        try:
            os.system("pip install openstudio")
        except Exception:
            os.system("pip install openstudio --user")
        try:
            import openstudio
            openstudio.Logger.instance().standardOutLogger().setLogLevel(openstudio.Fatal)
            return openstudio
        except Exception:
            warnings.warn("Energy - Error: Could not import openstudio. Please install it manually. Returning None.")
            return None


def _assets_dir():
    """
    Returns the EnergyModel assets directory (OSM template / EPW / DDY).

    Prefers the assets bundled with this package; falls back to a topologicpy
    install if it happens to ship them.
    """
    local = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets", "EnergyModel")
    if os.path.isdir(local):
        return local
    try:
        import topologicpy
        d = os.path.join(os.path.dirname(topologicpy.__file__), "assets", "EnergyModel")
        return d if os.path.isdir(d) else None
    except Exception:
        return None


class Energy:
    # ------------------------------------------------------------------ #
    # Geometry extraction from topologic-fast topologies                  #
    # ------------------------------------------------------------------ #
    @staticmethod
    def _cells(building):
        """Returns the list of Cells of the input building (Cell or CellComplex)."""
        import topologic_fast as tf
        if isinstance(building, tf.CellComplex):
            return list(building.Cells())
        if isinstance(building, tf.Cell):
            return [building]
        return None

    @staticmethod
    def _faces_of_cell(cell, mantissa=6):
        """Extracts a neutral face representation from a topologic-fast Cell."""
        faces = []
        for f in cell.Faces():
            boundary = f.ExternalBoundary()
            if boundary is None:
                continue
            verts = [tuple(round(c, mantissa) for c in v.Coordinates()) for v in boundary.Vertices()]
            if len(verts) < 3:
                continue
            faces.append({
                "vertices": verts,
                "normal": tuple(float(c) for c in f.Normal()),
                "com": tuple(round(c, mantissa) for c in f.CenterOfMass()),
            })
        return faces

    @staticmethod
    def _building_to_spaces(building, mantissa=6):
        """
        Converts a topologic-fast building into a neutral list of spaces, each a
        dict with ``com_z``, ``volume`` and a list of faces. Faces shared between
        two cells (matching centre-of-mass) are flagged ``is_exterior=False``.
        """
        cells = Energy._cells(building)
        if cells is None:
            return None
        spaces = []
        com_counts = {}
        for cell in cells:
            faces = Energy._faces_of_cell(cell, mantissa=mantissa)
            for fc in faces:
                com_counts[fc["com"]] = com_counts.get(fc["com"], 0) + 1
            com = cell.CenterOfMass()
            spaces.append({
                "com": (round(com[0], mantissa), round(com[1], mantissa), round(com[2], mantissa)),
                "com_z": round(com[2], mantissa),
                "volume": cell.Volume(),
                "faces": faces,
            })
        for sp in spaces:
            for fc in sp["faces"]:
                fc["is_exterior"] = com_counts.get(fc["com"], 0) <= 1
        return spaces

    @staticmethod
    def _floor_levels(building, mantissa=6, tolerance=0.0001):
        """Derives floor-level Z heights from the building's horizontal faces."""
        cells = Energy._cells(building)
        if cells is None:
            return None
        levels = set()
        for cell in cells:
            for f in cell.Faces():
                n = f.Normal()
                if abs(abs(n[2]) - 1.0) <= 1e-3:  # horizontal face
                    levels.add(round(f.CenterOfMass()[2], mantissa))
        return sorted(levels)

    # ------------------------------------------------------------------ #
    # Model creation                                                      #
    # ------------------------------------------------------------------ #
    @staticmethod
    def ByTopology(building,
                   shadingSurfaces=None,
                   osModelPath=None,
                   weatherFilePath=None,
                   designDayFilePath=None,
                   floorLevels=None,
                   buildingName="TopologicBuilding",
                   buildingType="Commercial",
                   northAxis=0.0,
                   glazingRatio=0.0,
                   coolingTemp=25.0,
                   heatingTemp=20.0,
                   defaultSpaceType="189.1-2009 - Office - WholeBuilding - Lg Office - CZ4-8",
                   mantissa=6,
                   tolerance=0.0001):
        """
        Creates an OpenStudio energy model from a topologic-fast building.

        Parameters
        ----------
        building : Cell or CellComplex
            The input building topology.
        shadingSurfaces : Topology , optional
            A topology (with ``.Faces()``) providing shading surfaces. Default None.
        osModelPath, weatherFilePath, designDayFilePath : str , optional
            Paths to the OSM template / EPW weather / DDY design-day files.
            Default (None) uses topologicpy's bundled assets.
        floorLevels : list , optional
            Floor-level Z heights. Default (None) derives them from horizontal faces.
        buildingName : str , optional
            The building name. Default "TopologicBuilding".
        buildingType : str , optional
            The standards building type. Default "Commercial".
        northAxis : float , optional
            North axis angle in degrees. Default 0.0.
        glazingRatio : float , optional
            Window-to-wall ratio for exterior walls. Default 0.0 (no windows).
        coolingTemp, heatingTemp : float , optional
            Thermostat setpoints in degrees C. Defaults 25.0 / 20.0.
        defaultSpaceType : str , optional
            The space type name to apply to all spaces.
        mantissa : int , optional
            Decimal places for coordinate rounding. Default 6.
        tolerance : float , optional
            Geometric tolerance. Default 0.0001.

        Returns
        -------
        openstudio.openstudiomodelcore.Model
            The created OpenStudio model, or None on failure.
        """
        openstudio = _openstudio()
        if openstudio is None:
            return None

        def safe_optional_get(opt):
            try:
                if opt and opt.is_initialized():
                    return opt.get()
            except Exception:
                pass
            return None

        def safe_name(model_object, fallback=""):
            try:
                name_opt = model_object.name()
                if name_opt.is_initialized():
                    return name_opt.get()
            except Exception:
                pass
            return fallback

        def surface_tilt_degrees(osSurface):
            try:
                up = openstudio.Vector3d(0, 0, 1)
                dot = osSurface.outwardNormal().dot(up)
                dot = max(-1.0, min(1.0, dot))
                return math.degrees(math.acos(dot))
            except Exception:
                return 90.0

        def vertices_to_point3d_list(vertices):
            return [openstudio.Point3d(v[0], v[1], v[2]) for v in vertices]

        def orient_surface_vertices(points, face_normal, surface_obj):
            try:
                osFaceNormal = openstudio.Vector3d(face_normal[0], face_normal[1], face_normal[2])
                osFaceNormal.normalize()
                if osFaceNormal.dot(surface_obj.outwardNormal()) < 1e-6:
                    surface_obj.setVertices(list(reversed(points)))
            except Exception:
                pass

        def os_path(path_str):
            try:
                return openstudio.openstudioutilitiescore.toPath(path_str)
            except Exception:
                pass
            try:
                return openstudio.toPath(path_str)
            except Exception:
                pass
            return openstudio.path(path_str)

        # ---- resolve assets -------------------------------------------------
        assets = _assets_dir()
        if not osModelPath:
            if assets is None:
                raise FileNotFoundError("Energy.ByTopology - Could not locate OSM template assets (topologicpy not installed).")
            osModelPath = os.path.join(assets, "OSMTemplate-OfficeBuilding-3.10.0.osm")
        if not weatherFilePath:
            weatherFilePath = os.path.join(assets, "GBR_London.Gatwick.037760_IWEC.epw")
        if not designDayFilePath:
            designDayFilePath = os.path.join(assets, "GBR_London.Gatwick.037760_IWEC.ddy")
        for p in (osModelPath, weatherFilePath, designDayFilePath):
            if not os.path.exists(p):
                raise FileNotFoundError(f"Energy.ByTopology - Required file not found: {p}")

        # ---- validate building ---------------------------------------------
        spaces_rep = Energy._building_to_spaces(building, mantissa=mantissa)
        if spaces_rep is None:
            warnings.warn("Energy.ByTopology - Error: building must be a Cell or CellComplex. Returning None.")
            return None

        # ---- load template, weather, design days ---------------------------
        translator = openstudio.osversion.VersionTranslator()
        model_opt = translator.loadModel(os_path(osModelPath))
        if (not model_opt) or (not model_opt.is_initialized()):
            raise RuntimeError(f"Energy.ByTopology - Could not load OSM template: {osModelPath}")
        osModel = model_opt.get()

        epw_opt = openstudio.openstudioutilitiesfiletypes.EpwFile.load(os_path(weatherFilePath))
        if epw_opt.is_initialized():
            openstudio.model.WeatherFile.setWeatherFile(osModel, epw_opt.get())
        else:
            raise RuntimeError(f"Energy.ByTopology - Could not load EPW weather file: {weatherFilePath}")

        ddy_opt = openstudio.openstudioenergyplus.loadAndTranslateIdf(os_path(designDayFilePath))
        if ddy_opt.is_initialized():
            for ddy in ddy_opt.get().getObjectsByType(openstudio.IddObjectType("OS:SizingPeriod:DesignDay")):
                osModel.addObject(ddy.clone())
        else:
            raise RuntimeError(f"Energy.ByTopology - Could not load DDY design day file: {designDayFilePath}")

        # ---- space type ----------------------------------------------------
        space_type_names = Energy.SpaceTypeNames(osModel)
        if defaultSpaceType is None:
            for stn in space_type_names:
                if "office" in stn.lower() or "room" in stn.lower():
                    defaultSpaceType = stn
                    break
        if defaultSpaceType not in space_type_names:
            raise RuntimeError(f"Energy.ByTopology - Default Space Type {defaultSpaceType} not found in OSM template.")

        osBuilding = osModel.getBuilding()

        if not floorLevels:
            floorLevels = Energy._floor_levels(building, mantissa=mantissa, tolerance=tolerance)
        if not floorLevels or len(floorLevels) < 2:
            raise RuntimeError("Energy.ByTopology - Could not derive valid floor levels from the input topology.")
        floorLevels = sorted(list(set(floorLevels)))
        numberOfStories = len(floorLevels) - 1
        if numberOfStories < 1:
            raise RuntimeError("Energy.ByTopology - The derived number of stories is less than 1.")

        osBuilding.setStandardsNumberOfStories(numberOfStories)
        floor_to_floor_height = (max(floorLevels) - min(floorLevels)) / numberOfStories
        if floor_to_floor_height <= tolerance:
            floor_to_floor_height = 3.0
        osBuilding.setNominalFloortoFloorHeight(floor_to_floor_height)

        defaultConstructionSets = list(osModel.getDefaultConstructionSets())
        if len(defaultConstructionSets) < 1:
            raise RuntimeError("Energy.ByTopology - No DefaultConstructionSet found in OSM template.")
        osBuilding.setDefaultConstructionSet(defaultConstructionSets[0])

        defaultScheduleSets = list(osModel.getDefaultScheduleSets())
        if len(defaultScheduleSets) < 1:
            raise RuntimeError("Energy.ByTopology - No DefaultScheduleSet found in OSM template.")
        osBuilding.setDefaultScheduleSet(defaultScheduleSets[0])

        osBuilding.setName(buildingName)
        try:
            osBuilding.setStandardsBuildingType(buildingType)
        except Exception:
            pass

        defaultSpaceTypeOpt = osModel.getSpaceTypeByName(defaultSpaceType)
        if not defaultSpaceTypeOpt.is_initialized():
            raise RuntimeError(f"Energy.ByTopology - Could not find SpaceType '{defaultSpaceType}' in the OSM template.")
        defaultSpaceTypeObj = defaultSpaceTypeOpt.get()
        osBuilding.setSpaceType(defaultSpaceTypeObj)

        for storyNumber in range(numberOfStories):
            osBuildingStory = openstudio.model.BuildingStory(osModel)
            osBuildingStory.setName("STORY_" + str(storyNumber))
            osBuildingStory.setNominalZCoordinate(floorLevels[storyNumber])
            osBuildingStory.setNominalFloortoFloorHeight(floor_to_floor_height)

        try:
            osBuilding.setNorthAxis(northAxis)
        except Exception:
            pass

        heatingScheduleConstant = openstudio.model.ScheduleConstant(osModel)
        heatingScheduleConstant.setValue(heatingTemp)
        coolingScheduleConstant = openstudio.model.ScheduleConstant(osModel)
        coolingScheduleConstant.setValue(coolingTemp)

        osThermostat = openstudio.model.ThermostatSetpointDualSetpoint(osModel)
        osThermostat.setHeatingSetpointTemperatureSchedule(heatingScheduleConstant)
        osThermostat.setCoolingSetpointTemperatureSchedule(coolingScheduleConstant)

        osBuildingStorys = list(osModel.getBuildingStorys())
        osBuildingStorys.sort(key=lambda x: safe_optional_get(x.nominalZCoordinate())
                              if safe_optional_get(x.nominalZCoordinate()) is not None else -1e12)
        if len(osBuildingStorys) < 1:
            raise RuntimeError("Energy.ByTopology - No BuildingStory objects were created.")

        interiorHorizontalConstruction = None
        try:
            dcs = defaultConstructionSets[0]
            isc_opt = dcs.defaultInteriorSurfaceConstructions()
            if isc_opt.is_initialized():
                isc = isc_opt.get()
                if isc.floorConstruction().is_initialized():
                    interiorHorizontalConstruction = isc.floorConstruction().get()
                elif isc.roofCeilingConstruction().is_initialized():
                    interiorHorizontalConstruction = isc.roofCeilingConstruction().get()
        except Exception:
            interiorHorizontalConstruction = None

        # ---- spaces & surfaces ---------------------------------------------
        osSpaces = []
        for spaceNumber, space_rep in enumerate(spaces_rep):
            osSpace = openstudio.model.Space(osModel)
            osSpaceZ = space_rep["com_z"]

            selectedStory = osBuildingStorys[0]
            for story in osBuildingStorys:
                storyZ = safe_optional_get(story.nominalZCoordinate())
                storyH = safe_optional_get(story.nominalFloortoFloorHeight())
                if storyZ is None:
                    continue
                if storyH is None:
                    storyH = floor_to_floor_height
                if storyZ + storyH < osSpaceZ:
                    continue
                if storyZ <= osSpaceZ:
                    selectedStory = story
                break
            osSpace.setBuildingStory(selectedStory)

            osSpaceName = "SPACE_" + "{:04d}".format(spaceNumber)
            osSpace.setName(osSpaceName)
            osSpace.setSpaceType(defaultSpaceTypeObj)

            space_com = space_rep["com"]
            for faceNumber, face_rep in enumerate(space_rep["faces"]):
                osFacePoints = vertices_to_point3d_list(face_rep["vertices"])
                osSurface = openstudio.model.Surface(osFacePoints, osModel)
                # Orient each surface outward from THIS cell's centroid rather than
                # trusting the face's stored winding: a face shared between two
                # cells keeps a single winding (its canonical owner's), so the two
                # cells must each derive their own outward sense for the interior
                # surfaces to match (opposite orientations) in OpenStudio.
                fcom = face_rep["com"]
                outward = (fcom[0] - space_com[0], fcom[1] - space_com[1], fcom[2] - space_com[2])
                mag = math.sqrt(outward[0] ** 2 + outward[1] ** 2 + outward[2] ** 2)
                if mag > tolerance:
                    outward = (outward[0] / mag, outward[1] / mag, outward[2] / mag)
                else:
                    outward = face_rep["normal"]
                orient_surface_vertices(osFacePoints, outward, osSurface)
                osSurface.setSpace(osSpace)

                tilt = surface_tilt_degrees(osSurface)
                space_name = safe_name(osSpace, f"SPACE_{spaceNumber:04d}")

                if face_rep["is_exterior"]:  # Exterior surfaces
                    osSurface.setOutsideBoundaryCondition("Outdoors")
                    if tilt > 135 or tilt < 45:
                        osSurface.setSurfaceType("RoofCeiling")
                        osSurface.setOutsideBoundaryCondition("Outdoors")
                        osSurface.setName(space_name + "_TopHorizontalSlab_" + str(faceNumber))
                        try:
                            face_zs = [v[2] for v in face_rep["vertices"]]
                            if len(face_zs) > 0 and max(face_zs) < 1e-6:
                                osSurface.setSurfaceType("Floor")
                                osSurface.setOutsideBoundaryCondition("Ground")
                                osSurface.setName(space_name + "_BottomHorizontalSlab_" + str(faceNumber))
                        except Exception:
                            pass
                    else:
                        osSurface.setSurfaceType("Wall")
                        osSurface.setOutsideBoundaryCondition("Outdoors")
                        osSurface.setName(space_name + "_ExternalVerticalFace_" + str(faceNumber))
                        if glazingRatio > 0.01:
                            try:
                                osSurface.setWindowToWallRatio(glazingRatio)
                            except Exception:
                                pass
                else:  # Interior surfaces
                    if tilt > 135:
                        osSurface.setSurfaceType("Floor")
                        osSurface.setName(space_name + "_InternalHorizontalFace_" + str(faceNumber))
                        if interiorHorizontalConstruction is not None:
                            try:
                                osSurface.setConstruction(interiorHorizontalConstruction)
                            except Exception:
                                pass
                    elif tilt < 40:
                        osSurface.setSurfaceType("RoofCeiling")
                        osSurface.setName(space_name + "_InternalHorizontalFace_" + str(faceNumber))
                        if interiorHorizontalConstruction is not None:
                            try:
                                osSurface.setConstruction(interiorHorizontalConstruction)
                            except Exception:
                                pass
                    else:
                        osSurface.setSurfaceType("Wall")
                        osSurface.setName(space_name + "_InternalVerticalFace_" + str(faceNumber))

            osThermalZone = openstudio.model.ThermalZone(osModel)
            cellVolume = space_rep["volume"]
            if cellVolume is not None:
                try:
                    osThermalZone.setVolume(cellVolume)
                except Exception:
                    pass
            osThermalZone.setName(osSpaceName + "_THERMAL_ZONE")
            osThermalZone.setUseIdealAirLoads(True)
            osThermalZone.setThermostatSetpointDualSetpoint(osThermostat)
            osSpace.setThermalZone(osThermalZone)

            for x in osSpaces:
                try:
                    if osSpace.boundingBox().intersects(x.boundingBox()):
                        osSpace.matchSurfaces(x)
                except Exception:
                    pass
            osSpaces.append(osSpace)

        # ---- shading surfaces ----------------------------------------------
        if shadingSurfaces is not None and hasattr(shadingSurfaces, "Faces"):
            osShadingGroup = openstudio.model.ShadingSurfaceGroup(osModel)
            for faceIndex, shadingFace in enumerate(shadingSurfaces.Faces()):
                boundary = shadingFace.ExternalBoundary()
                if boundary is None:
                    continue
                verts = [tuple(round(c, mantissa) for c in v.Coordinates()) for v in boundary.Vertices()]
                if len(verts) < 3:
                    continue
                facePoints = vertices_to_point3d_list(verts)
                aShadingSurface = openstudio.model.ShadingSurface(facePoints, osModel)
                faceNormal = tuple(float(c) for c in shadingFace.Normal())
                try:
                    osFaceNormal = openstudio.Vector3d(faceNormal[0], faceNormal[1], faceNormal[2])
                    osFaceNormal.normalize()
                    if osFaceNormal.dot(aShadingSurface.outwardNormal()) < 0:
                        aShadingSurface.setVertices(list(reversed(facePoints)))
                except Exception:
                    pass
                aShadingSurface.setName("SHADINGSURFACE_" + str(faceIndex))
                aShadingSurface.setShadingSurfaceGroup(osShadingGroup)

        osModel.purgeUnusedResourceObjects()
        return osModel

    # ------------------------------------------------------------------ #
    # Model utilities (ports of the corresponding EnergyModel methods)    #
    # ------------------------------------------------------------------ #
    @staticmethod
    def SpaceTypes(model):
        """Returns the space types in the input OSM model."""
        return list(model.getSpaceTypes())

    @staticmethod
    def SpaceTypeNames(model):
        """Returns the list of space-type names in the input OSM model."""
        names = []
        for st in model.getSpaceTypes():
            try:
                n = st.name()
                if n.is_initialized():
                    names.append(n.get())
            except Exception:
                pass
        return names

    @staticmethod
    def DefaultConstructionSets(model):
        """Returns ``[sets, names]`` for the default construction sets."""
        sets = model.getDefaultConstructionSets()
        return [list(sets), [s.name().get() for s in sets]]

    @staticmethod
    def DefaultScheduleSets(model):
        """Returns ``[sets, names]`` for the default schedule sets."""
        sets = model.getDefaultScheduleSets()
        return [list(sets), [s.name().get() for s in sets]]

    @staticmethod
    def ExportToOSM(model, path, overwrite=False):
        """
        Exports the input OSM model to an .osm file.

        Parameters
        ----------
        model : openstudio.openstudiomodelcore.Model
            The input OSM model.
        path : str
            The output path.
        overwrite : bool , optional
            If True, overwrite an existing file. Default False.

        Returns
        -------
        bool
            True if written successfully.
        """
        openstudio = _openstudio()
        if openstudio is None:
            return None
        if path[-4:].lower() != ".osm":
            path = path + ".osm"
        if not overwrite and os.path.exists(path):
            print("Energy.ExportToOSM - Error: file exists and overwrite is False. Returning None.")
            return None
        return model.save(path, overwrite)

    @staticmethod
    def GBXMLString(model):
        """Returns the gbXML string of the input OSM model."""
        openstudio = _openstudio()
        if openstudio is None:
            return None
        return openstudio.gbxml.GbXMLForwardTranslator().modelToGbXMLString(model)

    @staticmethod
    def Run(model, weatherFilePath=None, osBinaryPath=None, outputFolder=None, removeFiles=False):
        """
        Runs an EnergyPlus simulation for the input OSM model.

        Parameters
        ----------
        model : openstudio.openstudiomodelcore.Model
            The input OSM model.
        weatherFilePath : str , optional
            The EPW weather file. Default (None) uses topologicpy's bundled EPW.
        osBinaryPath : str , optional
            Path to the OpenStudio CLI binary. Default (None) attempts to locate it.
        outputFolder : str , optional
            Output directory. Default (None) uses a temp directory next to the model.
        removeFiles : bool , optional
            If True, remove generated files after extracting the SQL result.

        Returns
        -------
        openstudio.openstudiomodelcore.Model
            The model with its ``sqlFile`` attached, or None on failure.
        """
        import shutil
        from datetime import datetime, timezone
        openstudio = _openstudio()
        if openstudio is None:
            return None

        assets = _assets_dir()
        if weatherFilePath is None and assets is not None:
            weatherFilePath = os.path.join(assets, "GBR_London.Gatwick.037760_IWEC.epw")
        if osBinaryPath is None:
            osBinaryPath = shutil.which("openstudio")
        if osBinaryPath is None:
            warnings.warn("Energy.Run - Error: Could not locate the OpenStudio CLI binary. Returning None.")
            return None
        if outputFolder is None:
            stamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S_%f")
            outputFolder = os.path.join(os.getcwd(), "energy_run_" + stamp)
        os.makedirs(outputFolder, exist_ok=True)

        osmPath = os.path.join(outputFolder, "model.osm")
        model.save(osmPath, True)

        workflow = model.workflowJSON()
        try:
            workflow.setSeedFile(openstudio.path(osmPath))
            workflow.setWeatherFile(openstudio.path(weatherFilePath))
            workflow.saveAs(openstudio.path(os.path.join(outputFolder, "workflow.osw")))
        except Exception:
            pass

        oswPath = os.path.join(outputFolder, "workflow.osw")
        cmd = f'"{osBinaryPath}" run -w "{oswPath}"'
        os.system(cmd)

        sqlPath = os.path.join(outputFolder, "run", "eplusout.sql")
        if not os.path.exists(sqlPath):
            warnings.warn("Energy.Run - Error: simulation did not produce a SQL output file. Returning None.")
            return None
        sqlFile = openstudio.SqlFile(openstudio.path(sqlPath))
        model.setSqlFile(sqlFile)
        if removeFiles:
            try:
                shutil.rmtree(outputFolder)
            except Exception:
                pass
        return model

    @staticmethod
    def SqlFile(model):
        """Returns the SqlFile attached to the model, or None."""
        try:
            opt = model.sqlFile()
            return opt.get() if opt.is_initialized() else None
        except Exception:
            return None

    @staticmethod
    def Query(model,
              reportName="HVACSizingSummary",
              reportForString="Entire Facility",
              tableName="Zone Sensible Cooling",
              columnName="Calculated Design Load",
              rowNames=None,
              units="W"):
        """
        Queries the model's SQL results for tabular values.

        Parameters
        ----------
        model : openstudio.openstudiomodelcore.Model
            The input OSM model (must have an attached SQL file, see Run).
        reportName, reportForString, tableName, columnName : str
            The tabular-data selectors.
        rowNames : list , optional
            The row names to query. Default (None) queries all rows.
        units : str , optional
            The units string. Default "W".

        Returns
        -------
        list
            The queried values.
        """
        sqlFile = Energy.SqlFile(model)
        if sqlFile is None:
            warnings.warn("Energy.Query - Error: model has no SQL file. Run the model first. Returning None.")
            return None
        if rowNames is None:
            rowNames = Energy.RowNames(model, reportName, tableName)
        values = []
        for rowName in rowNames:
            query = ("SELECT Value FROM tabulardatawithstrings WHERE ReportName='" + reportName +
                     "' AND ReportForString='" + reportForString + "' AND TableName='" + tableName +
                     "' AND RowName='" + rowName + "' AND ColumnName='" + columnName + "'")
            if units:
                query += " AND Units='" + units + "'"
            result = sqlFile.execAndReturnFirstDouble(query)
            values.append(result.get() if result.is_initialized() else None)
        return values

    @staticmethod
    def ReportNames(model):
        """Returns the list of report names available in the model's SQL file."""
        sqlFile = Energy.SqlFile(model)
        if sqlFile is None:
            return None
        query = "SELECT ReportName FROM tabulardatawithstrings"
        names = sqlFile.execAndReturnVectorOfString(query)
        if not names.is_initialized():
            return []
        from collections import OrderedDict
        return list(OrderedDict((x, 1) for x in names.get()).keys())

    @staticmethod
    def TableNames(model, reportName):
        """Returns the table names for the given report in the model's SQL file."""
        sqlFile = Energy.SqlFile(model)
        if sqlFile is None:
            return None
        query = "SELECT TableName FROM tabulardatawithstrings WHERE ReportName='" + reportName + "'"
        names = sqlFile.execAndReturnVectorOfString(query)
        if not names.is_initialized():
            return []
        from collections import OrderedDict
        return list(OrderedDict((x, 1) for x in names.get()).keys())

    @staticmethod
    def RowNames(model, reportName, tableName):
        """Returns the row names for the given report/table in the model's SQL file."""
        sqlFile = Energy.SqlFile(model)
        if sqlFile is None:
            return None
        query = ("SELECT RowName FROM tabulardatawithstrings WHERE ReportName='" + reportName +
                 "' AND TableName='" + tableName + "'")
        names = sqlFile.execAndReturnVectorOfString(query)
        if not names.is_initialized():
            return []
        from collections import OrderedDict
        return list(OrderedDict((x, 1) for x in names.get()).keys())
