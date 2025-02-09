# OSM unit tests

Importing existing modal filters and turn restrictions from OSM can be very complicated. This directory contains minimal synthetic OSM inputs to test different situations.

## Creating a unit test

1.  Open the iD editor and zoom in on an empty patch of ocean, [like here](https://www.openstreetmap.org/edit#map=17/55.704282/-0.110917)
2.  Create the simplest example for some test case
3.  Press **Save** in the top-right corner
4.  Go to the bottom and **Download osmChange file**. Do **not** actually upload this edit!
5.  Turn the .osc into osm.xml and add it to this directory: `osmium cat ~/Downloads/changes.osc -o my_test_case.osm.xml`
6.  Add a test case in `mod.rs`, following examples there

## Viewing the OSM unit test inputs

TODO
