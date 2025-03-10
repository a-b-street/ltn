# Test cases

This directory has project files for areas with boundaries that've proven buggy in the past. Each file can be loaded from the title screen.

The `output` directory has GeoJSON output capturing:

- Roads detected as interior to the neighbourhood
- Cell boundaries
- Shortcuts per interior road
- Existing modal filters

The "unit" test in `backend/src/tests.rs` verifies this output doesn't change. When it does, we can manually check the diffs and commit the updated file to approve it. There are a few methods for manually diffing:

1.  Load the test input file in the [current version of the tool](https://a-b-street.github.io/ltn) and locally.
2.  Load the old and new test output files in [GeoDiffr](https://dabreegster.github.io/geodiffr).
3.  Use a JSON diff tool like [meld](https://en.wikipedia.org/wiki/Meld_(software)). It's difficult to understand reordered features or slightly changed coordinates, though.

## Notes per test case

There are several stages of "working" for each of these:

1.  Can the desired boundary be drawn?
2.  Do the interior / perimeter roads get inferred correctly?
3.  Do existing filters, shortcuts, cells, everything else all work?

- `bristol_west`: used in a previous consultation
  - At least at stage 2, using new planarizing snapper!
  - <https://play.abstreet.org/0.3.36/ltn.html?system/gb/bristol/maps/east.bin&--consultation=pt1> is reference for when it was working
- `bristol_east`: used in a previous consultation
  - The southern portion should stretch to the river
    - option 1: allow freehand points
    - option 2: include waterways in the snapper graph, not sure how connections will work
    - option 3: include footpaths in the snapper graph -- but we get lucky here
  - <https://play.abstreet.org/0.3.36/ltn.html?system/gb/bristol/maps/east.bin&--consultation=pt2> is reference for when it was working
- `strasbourg`: from <https://github.com/a-b-street/abstreet/issues/1006>
  - At least at stage 2

TODO:
- Lyon
- https://github.com/a-b-street/abstreet/issues/1065
- https://github.com/a-b-street/abstreet/issues/1087
- Go through emails
