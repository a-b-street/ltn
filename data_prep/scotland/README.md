This code is specific to Scotland, adapted from the Network Planning Tool project.

`boundaries.geojson` comes from <https://github.com/nptscot/npw/tree/main/data_prep>, turning polygons into multipolygons by doing `ogr2ogr boundaries.geojson -nlt PROMOTE_TO_MULTI path/to/npw/boundaries.geojson`
