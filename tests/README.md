# Test cases

This directory has project files for areas with boundaries that've proven buggy in the past. Eventually, we can also save the neighbourhood GeoJSON dump here too and have automated regression tests. Each file can be loaded from the title screen.

- `bristol_west`: used in a previous consultation
  - Working fine, using new planarizing snapper!
  - <https://play.abstreet.org/0.3.36/ltn.html?system/gb/bristol/maps/east.bin&--consultation=pt1> is reference for when it was working
- `bristol_east`: used in a previous consultation
  - the southern portion should stretch to the river; same freehand problem
  - <https://play.abstreet.org/0.3.36/ltn.html?system/gb/bristol/maps/east.bin&--consultation=pt2> is reference for when it was working
- `strasbourg`: from <https://github.com/a-b-street/abstreet/issues/1006>
  - Working fine!

TODO:
- Lyon
- https://github.com/a-b-street/abstreet/issues/1065
- https://github.com/a-b-street/abstreet/issues/1087
- Go through emails
