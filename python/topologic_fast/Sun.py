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
Solar geometry for topologic-fast.

This is a faithful port of ``topologicpy.Sun`` to the topologic-fast API. All
astronomical quantities are computed with the same `ephem` library and the same
calls used by topologicpy, so they are numerically identical. The geometric
helpers (Vector / Vertex / Edge / Path) reproduce topologicpy's
``Vector.ByAzimuthAltitude`` math exactly and build their results from
topologic-fast primitives.

Differences from topologicpy (documented for parity):
  * Path/diagram wires do not carry an attached ``Dictionary`` because
    topologic-fast does not expose topology dictionaries at the Python layer.
    The same metadata is returned separately by ``Diagram`` and is available on
    request via the ``*_meta`` helpers.
  * ``Diagram`` returns the sun-path wires (``date_paths`` / ``hourly_paths``);
    the purely decorative compass / shell / ground shapes are not generated.
"""
from __future__ import annotations

import math
import os
import warnings

def _ephem():
    """Imports and returns the ephem module, installing it if necessary."""
    try:
        import ephem
        return ephem
    except Exception:
        try:
            os.system("pip install ephem")
        except Exception:
            os.system("pip install ephem --user")
        try:
            import ephem
            return ephem
        except Exception:
            warnings.warn("Sun - Error: Could not import ephem. Please install it manually. Returning None.")
            return None


class Sun:
    # ------------------------------------------------------------------ #
    # Astronomy (delegates to ephem exactly like topologicpy)            #
    # ------------------------------------------------------------------ #
    @staticmethod
    def WinterSolstice(latitude, year=None):
        """
        Returns the winter solstice datetime for the input latitude and year.

        Parameters
        ----------
        latitude : float
            The input latitude.
        year : int , optional
            The input year. Default (None) uses the current year.

        Returns
        -------
        datetime
            The datetime of the winter solstice.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        from datetime import datetime
        if year is None:
            year = datetime.now().year
        if latitude >= 0:
            solstice = ephem.next_solstice(ephem.Date(f"{year}/12/1"))
        else:
            solstice = ephem.next_solstice(ephem.Date(f"{year}/6/1"))
        return solstice.datetime()

    @staticmethod
    def SummerSolstice(latitude, year=None):
        """
        Returns the summer solstice datetime for the input latitude and year.

        Parameters
        ----------
        latitude : float
            The input latitude.
        year : int , optional
            The input year. Default (None) uses the current year.

        Returns
        -------
        datetime
            The datetime of the summer solstice.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        from datetime import datetime
        if year is None:
            year = datetime.now().year
        if latitude >= 0:
            solstice = ephem.next_solstice(ephem.Date(f"{year}/6/1"))
        else:
            solstice = ephem.next_solstice(ephem.Date(f"{year}/12/1"))
        return solstice.datetime()

    @staticmethod
    def SpringEquinox(latitude, year=None):
        """
        Returns the spring (vernal) equinox datetime for the input latitude and year.

        Parameters
        ----------
        latitude : float
            The input latitude.
        year : int , optional
            The input year. Default (None) uses the current year.

        Returns
        -------
        datetime
            The datetime of the spring equinox.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        from datetime import datetime
        if year is None:
            year = datetime.now().year
        if latitude >= 0:
            equinox = ephem.next_equinox(ephem.Date(f"{year}/3/1"))
        else:
            equinox = ephem.next_equinox(ephem.Date(f"{year}/9/1"))
        return equinox.datetime()

    @staticmethod
    def AutumnEquinox(latitude, year=None):
        """
        Returns the autumnal equinox datetime for the input latitude and year.

        Parameters
        ----------
        latitude : float
            The input latitude.
        year : int , optional
            The input year. Default (None) uses the current year.

        Returns
        -------
        datetime
            The datetime of the autumn equinox.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        from datetime import datetime
        if year is None:
            year = datetime.now().year
        if latitude >= 0:
            equinox = ephem.next_equinox(ephem.Date(f"{year}/9/1"))
        else:
            equinox = ephem.next_equinox(ephem.Date(f"{year}/3/1"))
        return equinox.datetime()

    @staticmethod
    def Azimuth(latitude, longitude, date):
        """
        Returns the solar azimuth angle in degrees.

        Parameters
        ----------
        latitude : float
            The input latitude.
        longitude : float
            The input longitude.
        date : datetime
            The input datetime.

        Returns
        -------
        float
            The azimuth angle in degrees.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        observer = ephem.Observer()
        observer.date = date
        observer.lat = str(latitude)
        observer.lon = str(longitude)
        sun = ephem.Sun(observer)
        sun.compute(observer)
        return math.degrees(sun.az)

    @staticmethod
    def Altitude(latitude, longitude, date):
        """
        Returns the solar altitude angle in degrees.

        Parameters
        ----------
        latitude : float
            The input latitude.
        longitude : float
            The input longitude.
        date : datetime
            The input datetime.

        Returns
        -------
        float
            The altitude angle in degrees.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        observer = ephem.Observer()
        observer.date = date
        observer.lat = str(latitude)
        observer.lon = str(longitude)
        sun = ephem.Sun(observer)
        sun.compute(observer)
        return math.degrees(sun.alt)

    @staticmethod
    def Sunrise(latitude, longitude, date):
        """
        Returns the sunrise datetime for the input location and date.

        Parameters
        ----------
        latitude : float
            The input latitude.
        longitude : float
            The input longitude.
        date : datetime
            The input datetime (the date component is used).

        Returns
        -------
        datetime
            The sunrise datetime.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        date = date.replace(hour=12, minute=0, second=0, microsecond=0)
        observer = ephem.Observer()
        observer.lat = str(latitude)
        observer.lon = str(longitude)
        observer.date = date
        return observer.previous_rising(ephem.Sun()).datetime()

    @staticmethod
    def Sunset(latitude, longitude, date):
        """
        Returns the sunset datetime for the input location and date.

        Parameters
        ----------
        latitude : float
            The input latitude.
        longitude : float
            The input longitude.
        date : datetime
            The input datetime (the date component is used).

        Returns
        -------
        datetime
            The sunset datetime.
        """
        ephem = _ephem()
        if ephem is None:
            return None
        date = date.replace(hour=12, minute=0, second=0, microsecond=0)
        observer = ephem.Observer()
        observer.lat = str(latitude)
        observer.lon = str(longitude)
        observer.date = date
        return observer.next_setting(ephem.Sun()).datetime()

    # ------------------------------------------------------------------ #
    # Geometry helpers                                                    #
    # ------------------------------------------------------------------ #
    @staticmethod
    def _vector_by_azimuth_altitude(azimuth, altitude, north=0, reverse=False, mantissa=6):
        """
        Reproduces ``topologicpy.Vector.ByAzimuthAltitude`` exactly.

        The reference implementation builds the unit +Y vector, rotates it about
        the X axis by ``altitude`` and about the Z axis by ``-azimuth-north``,
        then optionally reverses it. The closed form below is algebraically
        identical and rounded to ``mantissa`` decimals to match topologicpy's
        ``Edge.Direction`` rounding.
        """
        alt_r = math.radians(altitude)
        b = math.radians(-azimuth - north)
        x = -math.cos(alt_r) * math.sin(b)
        y = math.cos(alt_r) * math.cos(b)
        z = math.sin(alt_r)
        # Mirror topologicpy's Edge.Direction, which normalizes the rotated edge
        # before rounding to `mantissa` decimals.
        mag = math.sqrt(x * x + y * y + z * z)
        if mag != 0:
            x, y, z = x / mag, y / mag, z / mag
        if reverse:
            x, y, z = -x, -y, -z
        return [round(x, mantissa), round(y, mantissa), round(z, mantissa)]

    @staticmethod
    def Vector(latitude, longitude, date, north=0):
        """
        Returns the sun as a unit vector pointing from the sun towards the origin.

        Parameters
        ----------
        latitude : float
            The input latitude.
        longitude : float
            The input longitude.
        date : datetime
            The input datetime.
        north : float , optional
            The compass angle of north in degrees (0 = +Y). Default is 0.

        Returns
        -------
        list
            The sun vector [x, y, z].
        """
        azimuth = Sun.Azimuth(latitude=latitude, longitude=longitude, date=date)
        altitude = Sun.Altitude(latitude=latitude, longitude=longitude, date=date)
        return Sun._vector_by_azimuth_altitude(azimuth=azimuth, altitude=altitude, north=north, reverse=True)

    @staticmethod
    def Vertex(latitude, longitude, date, origin=None, radius=0.5, north=0):
        """
        Returns the sun as a topologic-fast Vertex.

        Parameters
        ----------
        latitude : float
            The input latitude.
        longitude : float
            The input longitude.
        date : datetime
            The input datetime.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        north : float , optional
            The compass angle of north in degrees (0 = +Y). Default is 0.

        Returns
        -------
        Vertex
            The sun represented as a vertex.
        """
        from topologic_fast import Vertex as _Vertex, Topology as _Topology
        if origin is None:
            origin = _Vertex.Origin()
        # topologicpy: vector = Vector.Reverse(Sun.Vector(...)); translate origin
        # along that (normalized) direction by `radius`.
        vector = Sun._vector_by_azimuth_altitude(
            azimuth=Sun.Azimuth(latitude, longitude, date),
            altitude=Sun.Altitude(latitude, longitude, date),
            north=north, reverse=True,
        )
        vector = [-vector[0], -vector[1], -vector[2]]  # Vector.Reverse
        mag = math.sqrt(vector[0] ** 2 + vector[1] ** 2 + vector[2] ** 2)
        if mag == 0:
            mag = 1.0
        dx = vector[0] / mag * radius
        dy = vector[1] / mag * radius
        dz = vector[2] / mag * radius
        return _Topology.TranslateVertex(origin, dx, dy, dz)

    @staticmethod
    def Position(latitude, longitude, date, origin=None, radius=0.5, north=0, mantissa=6):
        """
        Returns the sun position as an [x, y, z] list.

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        date : datetime
            The input datetime.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        north : float , optional
            The compass angle of north in degrees. Default is 0.
        mantissa : int , optional
            Decimal places to round to. Default is 6.

        Returns
        -------
        list
            The sun position [x, y, z].
        """
        v = Sun.Vertex(latitude=latitude, longitude=longitude, date=date,
                       origin=origin, radius=radius, north=north)
        x, y, z = v.Coordinates()
        return [round(x, mantissa), round(y, mantissa), round(z, mantissa)]

    @staticmethod
    def Edge(latitude, longitude, date, origin=None, radius=0.5, north=0):
        """
        Returns the sun as an Edge pointing from the sun towards the origin.

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        date : datetime
            The input datetime.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        north : float , optional
            The compass angle of north in degrees. Default is 0.

        Returns
        -------
        Edge
            The sun edge (sun -> origin).
        """
        from topologic_fast import Vertex as _Vertex, Edge as _Edge
        if origin is None:
            origin = _Vertex.Origin()
        sun_v = Sun.Vertex(latitude=latitude, longitude=longitude, date=date,
                           origin=origin, radius=radius, north=north)
        return _Edge.ByStartVertexEndVertex(sun_v, origin)

    @staticmethod
    def VerticesByDate(latitude, longitude, date, startTime=None, endTime=None,
                       interval=60, origin=None, radius=0.5, north=0):
        """
        Returns the sun locations over a single day as a list of vertices.

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        date : datetime
            The input date.
        startTime : datetime , optional
            Start time. Default (None) uses sunrise.
        endTime : datetime , optional
            End time. Default (None) uses sunset.
        interval : int , optional
            Sampling interval in minutes. Default is 60.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        north : float , optional
            The compass angle of north in degrees. Default is 0.

        Returns
        -------
        list
            The sun locations as a list of vertices.
        """
        from datetime import timedelta
        if startTime is None:
            startTime = Sun.Sunrise(latitude=latitude, longitude=longitude, date=date)
        if endTime is None:
            endTime = Sun.Sunset(latitude=latitude, longitude=longitude, date=date)
        vertices = []
        current_time = startTime
        while current_time <= endTime:
            vertices.append(Sun.Vertex(latitude=latitude, longitude=longitude, date=current_time,
                                       origin=origin, radius=radius, north=north))
            current_time += timedelta(minutes=interval)
        return vertices

    @staticmethod
    def PathByDate(latitude, longitude, date, startTime=None, endTime=None, interval=60,
                   origin=None, radius=0.5, sides=None, north=0):
        """
        Returns the sun path over a single day as a Wire.

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        date : datetime
            The input date.
        startTime, endTime : datetime , optional
            Start/end time. Default (None) uses sunrise/sunset.
        interval : int , optional
            Sampling interval in minutes. Default is 60.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        sides : int , optional
            If set, the path is resampled into this many equal segments.
        north : float , optional
            The compass angle of north in degrees. Default is 0.

        Returns
        -------
        Wire
            The sun path as a (non-closed) wire, or None if fewer than 2 points.
        """
        from topologic_fast import Wire as _Wire
        vertices = Sun.VerticesByDate(latitude=latitude, longitude=longitude, date=date,
                                      startTime=startTime, endTime=endTime, interval=interval,
                                      origin=origin, radius=radius, north=north)
        if len(vertices) < 2:
            return None
        wire = _Wire.ByVertices(vertices, close=False)
        if sides is not None:
            vertices = [_Wire.VertexByParameter(wire, float(i) / float(sides)) for i in range(sides)]
            wire = _Wire.ByVertices(vertices, close=False)
        return wire

    @staticmethod
    def VerticesByHour(latitude, longitude, hour, startDay=1, endDay=365, interval=5,
                       origin=None, radius=0.5, north=0, year=None):
        """
        Returns the sun locations at a fixed hour across the year as vertices.

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        hour : float
            The input hour of day.
        startDay : int , optional
            Start day-of-year. Default is 1.
        endDay : int , optional
            End day-of-year. Default is 365.
        interval : int , optional
            Sampling interval in days. Default is 5.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        north : float , optional
            The compass angle of north in degrees. Default is 0.
        year : int , optional
            The year to use. Default (None) uses the current year. Exposed so
            results can be made deterministic / reproducible.

        Returns
        -------
        list
            The sun locations as a list of vertices.
        """
        from datetime import datetime, timedelta

        def day_of_year_to_datetime(yr, day_of_year):
            base_date = datetime(yr, 1, 1)
            return base_date + timedelta(days=day_of_year - 1)

        if year is None:
            year = datetime.now().year
        vertices = []
        for day_of_year in range(startDay, endDay, interval):
            date = day_of_year_to_datetime(year, day_of_year)
            date += timedelta(hours=hour)
            vertices.append(Sun.Vertex(latitude=latitude, longitude=longitude, date=date,
                                       origin=origin, radius=radius, north=north))
        return vertices

    @staticmethod
    def PathByHour(latitude, longitude, hour, startDay=1, endDay=365, interval=5,
                   origin=None, radius=0.5, sides=None, north=0, year=None):
        """
        Returns the analemma (fixed-hour sun path) across the year as a Wire.

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        hour : float
            The input hour of day.
        startDay, endDay : int , optional
            Day-of-year range. Defaults are 1 and 365.
        interval : int , optional
            Sampling interval in days. Default is 5.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        sides : int , optional
            If set, the path is resampled into this many equal (closed) segments.
        north : float , optional
            The compass angle of north in degrees. Default is 0.
        year : int , optional
            The year to use. Default (None) uses the current year.

        Returns
        -------
        Wire
            The analemma as a wire, or None if fewer than 2 points.
        """
        from topologic_fast import Wire as _Wire
        vertices = Sun.VerticesByHour(latitude=latitude, longitude=longitude, hour=hour,
                                      startDay=startDay, endDay=endDay, interval=interval,
                                      origin=origin, radius=radius, north=north, year=year)
        if len(vertices) < 2:
            return None
        wire = _Wire.ByVertices(vertices, close=False)
        if sides is not None:
            vertices = [_Wire.VertexByParameter(wire, float(i) / float(sides)) for i in range(sides)]
            wire = _Wire.ByVertices(vertices, close=True)
        return wire

    @staticmethod
    def Diagram(latitude, longitude, minuteInterval=30, dayInterval=15,
                origin=None, radius=0.5, uSides=180, vSides=180, north=0, year=None):
        """
        Returns the core sun-path geometry of a sun-path diagram.

        Unlike ``topologicpy.Sun.Diagram`` this returns only the sun-path wires
        (which carry all of the solar geometry); the purely decorative compass /
        shell / ground shapes are not generated (see module docstring).

        Parameters
        ----------
        latitude, longitude : float
            The input location.
        minuteInterval : int , optional
            Sampling interval (minutes) for the date paths. Default is 30.
        dayInterval : int , optional
            Sampling interval (days) for the hourly paths. Default is 15.
        origin : Vertex , optional
            The world origin. Default (None) is (0, 0, 0).
        radius : float , optional
            The radius of the sun orbit. Default is 0.5.
        uSides : int , optional
            Resampling resolution of each date path. Default is 180.
        vSides : int , optional
            Resampling resolution of each hourly path. Default is 180.
        north : float , optional
            The compass angle of north in degrees. Default is 0.
        year : int , optional
            The year to use. Default (None) uses the current year.

        Returns
        -------
        dict
            ``{'date_paths': [...], 'hourly_paths': [...], 'metadata': {...}}``.
        """
        from datetime import datetime, timedelta
        if year is None:
            year = datetime.now().year

        winter_solstice = Sun.WinterSolstice(latitude=latitude, year=year)
        summer_solstice = Sun.SummerSolstice(latitude=latitude, year=year)
        equinox = Sun.AutumnEquinox(latitude=latitude, year=year)

        date_paths = []
        for date in (winter_solstice, equinox, summer_solstice):
            startTime = Sun.Sunrise(latitude=latitude, longitude=longitude, date=date) - timedelta(hours=2)
            endTime = Sun.Sunset(latitude=latitude, longitude=longitude, date=date) + timedelta(hours=2)
            path = Sun.PathByDate(latitude=latitude, longitude=longitude, date=date,
                                  startTime=startTime, endTime=endTime, interval=minuteInterval,
                                  origin=origin, radius=radius, sides=uSides, north=north)
            date_paths.append(path)

        hourly_paths = []
        for hour in range(0, 24, 1):
            hourly_path = Sun.PathByHour(latitude, longitude, hour, startDay=1, endDay=365,
                                         interval=dayInterval, origin=origin, radius=radius,
                                         sides=vSides * 2, north=north, year=year)
            if hourly_path is not None:
                hourly_paths.append(hourly_path)

        return {
            "date_paths": date_paths,
            "hourly_paths": hourly_paths,
            "metadata": {"latitude": latitude, "longitude": longitude, "year": year, "north": north},
        }
