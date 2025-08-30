let e=[["This is the second version of the A/B Street LTN tool, ",[0,"an open source project"],". This version, called the Connected Neighbourhoods Tool, has been funded by ",[1,"Transport Scotland"]," and designed by ",[2,"Sustrans Scotland"],`.
The team includes:`],["Dustin Carlino, lead developer and director of ",[0,"A/B Street Ltd"]],[[0,"Michael Kirk"],`,
software developer`],["Sustrans Scotland ",[0,[0,[0,"Congying (Cici) Hu"],", mobility planner"]," ",[1,[0,"Angus Calder"],", senior mobility planner"]," ",[2,"Michael Naysmith, senior mobility planner (project manager)"]," ",[3,[0,"Matthew Davis"],", principal mobility planner"]]],[`This second version of the tool is an evolution from the original A/B Street
tool, with `,[0,"many people"],` to thank there. Of the dozens of users giving excellent feedback and shaping
the tool’s development, particular thanks to:`],"Arzhel Younsi","Rouelibre1","Jean-David Génevaux","This tool would not be possible without:","OpenStreetMap contributors","Maptiler","GeoRust","Lucide icons","And countless other open source libraries","Some road signs images © Crown copyright.","What do you want to name the neighbourhood?","Enter","Prioritisation",`Compare the prioritisation or individual metrics across candidate
neighbourhoods.`,"The colors are arbitrary, just to distinguish better.",[[0,[0]," Choose area"]," or ",[1,[0]," Draw area"]],"1. Click an area to start your neighbourhood",`These particular boundaries are suggested by finding roads,
railways, and water that form severances.`,"2. Include additional areas","Click any adjacent areas you'd like to add to your boundary.","1. Draw your neighbourhood","2. Refine your boundary",`Continue to add, delete, and move points until you're happy with
your neighbourhood boundary. Don't worry, you can always adjust the
boundary more later.`,"3. Finished?",`When you're done, click "Create".`,"Create neighborhood",[[0]," Start over"],"Your neighbourhood overall","Empty so far. Click a colored area to get started.",`Empty so far. Click the map to add points around your
neighborhood to get started.`,`Not a valid shape yet — try dragging the points into a box
around your neighbourhood. Note the ordering of the points.`,"Export metrics to GeoJSON","FeatureCollection",[[0,"Area:"]," ",0," km²"],[[0,"Car or van ownership"]," ",[1]," ",0," of approximately ",1," households have at least one car or van."],[[0,"Population density"]," ",[1]," ",0," people / km²"],[[0,"Pedestrian and cyclist collisions"]," ",[1]," ",0," / km²"],[[0,"Points of interest"]," ",[1]," ",0," / km²"],"Combined score","The Connected Neighbourhoods Tool","The Low-Traffic Neighbourhood (LTN) tool, v2","A/B Street logo","User guide","Zoom to fit study area","Escape","Back",["Trips begin and end in ",0," zones"],["Trips from ",[0]," Trips to"],["Trips ",0," ",1,":"],["Total trips from here: ",0],["Total trips to here: ",0],["Total intra-zonal trips starting and ending here: ",0],"Hover on a zone","Purple intersections have some kind of turn restriction.","Pick another intersection","Open OSM","Roads","Click a road to visit its OSM object.","Intersections","Turn restrictions:","Click an intersection to inspect its movements.",[0," routes cross here ",[0,"before your changes"]," , and ",1," ",[1,"after your changes"]," . That's ",2,"% of the original traffic."],[`The routes are currently sampled, to speed things up. This one sample
route represents `,0,` trips between the
same points.`],`Note: if these don't sum to the total above, that's likely a known
software bug`,["No possible route before changes ( ",[0,"This is usually a known software bug"]],["No possible route after changes ( ",[0,"This is usually a known software bug"]],"A","B",`This shows the change in driving time to one destination from everywhere
within the neighbourhood. Drag the pin around to change that destination.`,"Route before changes",["No possible route ( ",[0,"This is usually a known software bug"]," )"],"Hover on a road to compare","Route after changes",["Time ratio: ",0],"X","Calculating impact",[`This mode estimates the impact of all your changes on traffic around the
entire area. It's based on many assumptions and must be interpreted very
carefully. `,[0,"Explore the origin/destination demand data used"]],"About the 2011 home to work data",["This region uses preprocessed census data from the ",[0,"NPT project"],`. The data is very outdated. Trips to work are just one purpose,
representing a small percent of all trips and with strong spatial
biases.`],"About the 2021 home to work data",["This region uses the ",[0,"ODWP01EW"],` census dataset, with trips from home to work. It was taken during COVID-19,
so `,[1,"workplace travel patterns have major caveats"],`. Trips to work are just one purpose, representing a small percent
of all trips and with strong spatial biases. This dataset does not
distinguish by trip mode, so trips could be made by driving, public
transit, cycling, walking, etc.`],"About the fake origin/destination data",[`This region has no available origin/destination data about travel
patterns, so a `,[0,"completely random"],` small set of trips are
modelled. Contact Dustin at `,[1,"dabreegster@gmail.com"],` to set
up real data in your region.`],[[0]," Calculate quickly"],[[0]," Calculate more accurately"],`Red roads have increased traffic, and green roads have decreased. Thicker
roads have more traffic after edits. If hovering on a road doesn't show
anything, there was no change there. Click a road to see example routes
through it that've changed.`,["Only show roads with at least this many daily trips before or after ",[0]],[0," before, ",1," after"],"Drag markers for a route",["Slow-down factor for main roads: ",0," ",[0]],`Increase to see how drivers may detour in heavy traffic. 1 means
free-flow.`,"Adjust boundary","Finish","Cancel","No shortcuts here","Click a road to see shortcuts",`This shows all possible shortcuts crossing the blue road you've chosen.
A shortcut is defined as a route starting and ending on main (busy)
roads, then cutting through smaller streets. It might not actually be
considered a "good shortcut" in practice -- this tool doesn't know any
real traffic patterns; it's just looking for any possible path. This
view lets you understand the limits of this assumption.`,"Pick a different road",["This shortcut is ",[0,0,"x"]," the length of the shortest route using all roads, not just this neighbourhood"],[0," shortcuts through ",1],["Main road: ",0],"Help",["Area ",[0]],[0," km²"],["SIMD ",[0]],["quintile ",0],["Population density ",[0]],[0," people / km²"],["Car ownership ",[0]],[0," of households"],["POI density ",[0]],[0," / km²"],["Collision density ",[0]],["Overall prioritisation score ",[0]],"ArrowLeft","ArrowRight","Left","Previous","Right","Next","Street view",["Click the map to see street view ",[0]," ",[1]," ",[2,"Street view source"]," ",[3,[0]," Google Street View"]," ",[4,[0]," Bing Streetside"]],["Please name this copy of project ",0],["Project: ",0],"Download project as GeoJSON","Export","Make a copy of this project","Copy project","Open another project",[0," new modal filter(s) added"],[0," existing modal filter(s) removed"],[0," road segment direction(s) changed"],["Map data is from ",0],["If this area has changed since then, please contact ",[0,"dabreegster@gmail.com"]," to use newer OpenStreetMap data."],"Debug route-snapper",["Really delete neighbourhood ",0,"? You can't undo this."],["Rename neighbourhood ",0," to what?"],"Neighbourhoods","Rename neighbourhood","Delete neighbourhood",[[0]," Add a new neighbourhood"],`Compare the prioritisation or individual metrics across your
neighbourhoods.`,[[0,"Population density:"]," ",0,` people
/ km²`],[[0,"Pedestrian and cyclist collisions:"]," ",0," / km²"],[[0,"Points of interest:"]," ",0," / km²"],"Global version","Scotland","England",[`The LTN tool can be used anywhere in the world. Some regions have extra
datasets. If you'd like to specialize the tool, please email Dustin at `,[0,"dabreegster@gmail.com"],"."],["LAD_",0],"Choose a boundary below or on the map to begin:","Road labels",["Click to start a new project in ",0],"LAD_","Load a project from a file","Loading OSM",["Project name: ",[0]],["Load a built-in area: ",[0]],"or...",["Really delete project ",0,"? You can't undo this."],["Rename project ",0," to what?"],"The LTN Tool","Read the user guide","Your projects","Rename project","Delete project","Start a new project","New project","Waiting for MapLibre and WASM to load...","Choose a modal filter to place on streets","Confirm","Remove diagonal filter",[[0,"❌"]," ",[1]," Delete diagonal filter"],"Rotate diagonal filter",[[0,"↻"]," ",[1]," Rotate diagonal filter"],"Add diagonal filter",[[0,"+"]," ",[1]," Add diagonal filter"],[[0,"Click and drag"]," to move"],[[0,"Right click"]," to delete"],[[0,"Click"]," to reclassify everything highlighted"],"Bus routes",["These are all ",[0,"bus routes"]," according to OpenStreetMap."],"Existing cycling infrastructure","Data from NPT","Cycling safety Level of Service","Other","Estimated traffic","Traffic volume category","Level of Service","Infrastructure type","Mixed Traffic Street","Layers","Existing modal filters and turn restrictions","Metrics","Public transport integration","Active travel","Basemap","MapTiler Dataviz","MapTiler Streets","MapTiler Satellite","OS Open Zoomstack","Debugging tools","POIs",["See Scottish data sources for ",[0,"schools"]," , ",[1,"GP practices"]," , and ",[2,"hospitals"]," . Other data is from ",[3,"OpenStreetMap"]," ."],"SIMD","Less deprived","More deprived",["This shows the Scottish Index of Multiple Deprivation (SIMD) from ",[0,"2020 data"],` . SIMD combines different domains: income; employment; health; education, skills
and training; geographic access to services; crime; and housing.`],"Population density","Less dense","people / km²","More dense",["This shows population data from ",[0,"2020 data"]," ."],"Car ownership","Households with at least one car or van","Show households from the Scottish census with at least one car.",["Data zone ",0," has ",1," people, and a SIMD rank of ",2,`, making it less deprived
than `,3,"% of data zones."],["In data zone ",0," ",1," of approximately ",2," households have at least one car or van."],["Data zone ",0," has ",1," people, with a density of ",2," people per square kilometer"],"Railway stations","National Rail logo",[[0]," Station"],["These are all ",[0,"railway stations"]," according to OpenStreetMap."],"Unnamed railway station","Estimated cycling demand",["Trip purpose: ",[0]],["Scenario: ",[0]],["Network type: ",[0]],["Color by: ",[0]],["Cyclists: ",0],["Gradient: ",0,"%"],["Cycle-friendliness: ",0,"%"],"All network details","Fast/Direct network","Baseline","Go Dutch","E-bikes","Quiet/Indirect network","Collisions",["This layer shows collisions recorded in the ",[0,"DfT stats19"]," dataset, as of 30 September 2024. Please note these limitations:"],"Only collisions between 2017 and 2023 are included",`This tool is intended to be used when zoomed into the map, while
inspecting a scheme or development area. Not all points are shown when
zoomed out and showing large areas. Do not use this to look for trends
across a city or region scale.`,`Approximately 150 collisions from the source data aren't included, due
to problems with the recorded location`,'The "pedestrians" category also include mobility scooters',["All limitations ",[0,"documented by DfT"],` also apply. Not all collisions or near misses are reported. There's nuance
with the severity categories.`],"You can click a point to open the full report, thanks to CycleStreets.",["License: ",[0,"Open Government License"]," . Contains OS data © Crown copyright and database right 2025."],[[0]," Pedestrians"],[[0]," Cyclists"],["From ",[0]],["To ",[0]],["Year: ",[0]],["Severity: ",[0]],["Casualties: ",[0]],["Pedestrian location: ",[0]],["Pedestrian movement: ",[0]],"Click to open full report in CycleStreets","Metric","None","Area","Points of interest","Overall prioritisation score","km²","collisions / km²","POIs / km²","Least important","Most important","This score averages the other five metrics, on the 1-5 scale.","User guide for the Connected Neighbourhood Tool","Overview",[`The Connected Neighbourhood Tool helps you design a Low-Traffic
Neighbourhood (LTN). This guide is written specifically for the Scottish
version of the tool, but most of it applies elsewhere too. In this guide,
we assume you are already familiar with the concept and purpose of LTNs.
If you have any trouble using the tool, please email the maintainer at `,[0,"dabreegster@gmail.com"]," or ",[1,"create a Github issue"],"."],"The overall process of using the CNT looks like this:","Choose your study area","Add one or more neighbourhood boundaries",`Design your LTN with modal filters, one-way streets, turn restrictions,
and sometimes by reclassifying main roads`,"Explore the effects of your proposal","Share your work with a colleague","Adding a neighbourhood",["Start using the tool by going to ",[0,"https://cnt.scot/"],` and picking your study area on the map or from the list. The study areas are
defined by Local Authority Districts.`],`To design an LTN, you first need to specify its boundary. Unless you’re
working on a large circulation plan, the neighbourhood boundary will
probably be a much smaller area than the entire study area shown. You can
create multiple LTNs in one project, but you only need one to start.`,"Quick boundaries from severances",`In some cases, the boundary you want will already be shown on the map as a
coloured area. These areas are found automatically by dividing settlements
on the map by severances – main roads, railways, and bodies of water.`,`After clicking one area, you can keep clicking adjacent areas to extend
the boundary, in case the first boundary is too small.`,"Drawing manually",`Alternatively, you can draw an area in more detail by picking at least
three points on the map.`,`You can drag any of the red or grey points to adjust the boundary. Any
point you drag becomes a red waypoint:`,`The red points snap the boundary to roads. Sometimes near a park or body
of water without any roads, you may wish to draw the boundary in even more
detail by turning off snapping. Click any red point to turn it blue, which
you can drag anywhere you like:`,`When you draw a boundary manually, sometimes the resulting area doesn’t
have a valid shape:`,`The 1st and 2nd point form long line-like “spurs” away from the area.
Between red snapped points, the tool is trying to find the shortest
distance path along roads. Sometimes that path on both sides of a point
will use exactly the same roads, resulting in this spur. When you see this
happen, you can keep dragging points around, introducing more points, and
so on to fix the shape to match whatever you intend.`,"Prioritisation metrics","This feature is currently available in Scotland only.",[[0,"Transport for London’s Strategic Neighbourhood Analysis"],` describes an approach for prioritising LTNs by different metrics. The CNT
exposes some of these metrics for the areas:`],"Population density – generally LTNs have greater benefit in denser areas",["Collisions – using ",[0,"stats19 data"],` about prior collisions involving pedestrians and cyclists, it may be important
to target areas with existing problems`],`SIMD (Scottish Index of Multiple Deprivation) – depending on local
priorities, areas with higher deprivation may be important`,`Points of interest – areas with a mix of residential and commercial land
use can be important to improve walking and cycling`,`Car ownership – residents in areas with low car ownership are not
benefitting from through-traffic`,`Depending on local priorities, you may want to use some combination of
these metrics to decide where to prioritise creating an LTN. You can
colour the areas by any of these metrics:`,`As you select or draw a boundary, all of these metrics are evaluated
against your area:`,"Designing an LTN",`After specifying a neighbourhood boundary, you are in the main editing
mode. There are four editing controls available, but first you need to
understand the cells and shortcuts shown on the map.`,"Understanding the map: cells",[`This example neighbourhood is bounded on all sides by a grey main road,
where we assume the road is designed to handle a higher volume of traffic.
The smaller coloured areas inside the neighbourhood are `,[0,"cells"],`, showing internal connectivity for a driver. If a driver enters the
neighbourhood by the blue arrow, they are only able to reach the area
shown in blue; they can’t drive to the yellow or pink cells without
exiting back onto the main road, then re-entering the neighbourhood
somewhere else.`],`Another example is shown below. The orange cell is effectively a small
cul-de-sac; a driver won’t enter unless their journey starts or ends
there. They can’t access the larger blue cell.`,`Aside from these smaller cells, this neighbourhood mainly consists of the
large blue cell. There are many points where a driver can enter and exit
this cell. Because the blue cell stretches so far, a driver can enter from
the south and drive all the way through to the north.`,`If understanding the cells as areas is confusing or inconvenient, you can
modify the map style and colour the roads by their cell instead:`,"Understanding the map: shortcuts",[`To design an effective LTN, you must limit the traffic cutting through the
neighbourhood. `,[0,"Shortcuts"],` show the possible routes through
a neighbourhood a driver might take when avoiding main roads. They do not include
journeys starting or ending somewhere in the neighbourhood, just routes that
pass through without stopping. These are shown in shades of red; the white
streets are dead-ends and cul-de-sacs; a driver has no reason to go there unless
their trip starts or ends there. The darkest reds show the streets most likely
to see lots of traffic cutting through. The darkest red is along the long north/south
street:`],`To understand why, you can use the Shortcuts tool at the top. If we
inspect this street, we see one example shortcut from north to south:`,`The tool identifies 51 different shortcuts passing through this one
street, showing the most advantageous shortcuts first – the ones that save
the driver the most time by cutting through the middle of the
neighbourhood. Most of the shortcuts are simple variations, changing the
exact entrance or exit. There are also some shortcuts involving the
western boundary:`,"The tool counts these shortcuts in a simple way:","Find all entrances and exits onto streets from the main roads",`For every combination, calculate the fastest driving route, using the
speed limit and length of each road. Main roads are penalised as having
half their speed limit, to simulate delays in heavy traffic conditions.`,"Any route that crosses a main road is discarded","Count the number of routes crossing each street segment",[`The tool assumes a driver is equally likely to enter and exit the
neighbourhood through any point, but of course this doesn’t reflect the
real traffic patterns in the larger area. Maybe the northern boundary of
this neighbourhood isn’t attractive for drivers, because there’s no reason
to drive that way. (In this case, since the neighbourhood is just north of
Aberdeen city centre and the north/south shortcut is parallel to an A
road, it `,[0,"is"],` likely a shortcut that happens in practice.)
The tool’s assumptions are necessary to make due to a lack of detailed
traffic pattern data, and because they can be calculated even as you start
to edit the neighbourhood. The shortcuts simply show what is `,[1,"possible"],` for drivers to do, not what is likely. You may need
to apply your own local knowledge, judgment, or traffic counters to verify
a shortcut is actually a problem in practice.`],"Editing: modal filters",[`Now that you understand shortcuts, let’s move on to the interventions you
can propose to fix these problems. The main tool is the `,[0,"modal filter"],`, or point closure. It stops drivers from passing through a street, while
still allowing pedestrians and cyclists (and sometimes buses, emergency
vehicles, etc) through. Let’s try adding a modal filter along the
north/south shortcut:`],`Immediately after you click to add the filter, you’ll see the red
shortcuts jump to the right, zig-zagging to avoid the new filter. If you
add a second filter there, you’ll see a big change:`,`The blue cell has been split into a new yellow cell, making it clear that
now the north/south shortcut is totally impossible.`,`You may have noticed the modal filter icons on the map are different.
There are four types you can choose from:`,`In the scope of the tool, these all mean the same thing – a driver cannot
pass through. You can use the different types to communicate more specific
proposals. School streets are timed closures, but the tool will model the
effects of the filter during school hours. When you place a filter on a
street that currently has a bus route along it, you will automatically get
a bus gate, which uses camera enforcement and doesn’t physically prevent
vehicles from crossing. The specifics of the physical intervention are
outside the scope of this tool – depending on width constraints, allowing
adequate room for bin lorries to turn, and so on, the physical
implementation of a filter could be a pocket park, removable bollards,
concrete, etc. The LTN tool’s purpose is to focus on the strategic
planning.`,"Editing: diagonal modal filters",`Modal filters usually apply at one point along a street, but when you have
a four-way intersection, you can click it to toggle through two possible
diagonal filters. These allow traffic through the intersection only for
some combinations of streets.`,"Editing: one-way streets",`You can also change the direction of traffic flow along a street. This is
helpful to retain through-traffic in one direction, but funnel it back out
to a main road. Or sometimes a shortcut is only problematic in one
direction.`,`You cannot create new cells only by introducing one-way streets, but you
can influence shortcuts.`,"Editing: turn restrictions",`You can restrict some turns through an intersection without outright
preventing all movement. This may be useful to prevent unprotected turns
to or from a main road when there is no room for a turning lane.`,`Note that existing turn restrictions are automatically added from
OpenStreetMap data. There are some complex situations near dual
carriageways that may not be detected correctly; please contact the team
to report this problem if you encounter one.`,"Editing: main road classification",[`When you initially create a neighbourhood from its boundary, some roads
count as `,[0,"main roads"],`, shown in grey. The initial
classification is taken from OpenStreetMap data. Main roads are intended
to handle through-traffic, and so the tool does not calculate shortcuts
along main roads, and the cells are determined by connections to main
roads. In the example below, there are main roads surrounding the
perimeter of the neighbourhood, which is typical, but there are also two
north/south main roads in the middle, causing there to be cells on each
side.`],[`You may want to reclassify these main roads, and treat them like
residential streets that should not carry through-traffic. This could make
sense in the context of a larger circulation plan, a redesign to the
strategic road network in the wider area, or when the main road is a high
street with heavy foot and cycling traffic. No matter the reason, you can
mark new main roads or erase main roads using one of the tools. In complex
areas, it may be simplest to first `,[0,"Erase all main roads"]," and then ",[1,"Mark as main along a route"],`. After removing those two interior main
roads, the neighbourhood looks like one big cell:`],`You can now make other edits and see the effects on cells and shortcuts
through the entire area.`,"Exploring effects",`As you design an LTN, you are already understanding the effects on traffic
through the area, by paying attention to cells and shortcuts. You can also
study the effects on the entire study area.`,"Effect on one journey",`A common concern during public consultations is that a driving route that
previously cut through a neighbourhood will become much longer or
impossible after an LTN is created. You can use the route tool to evaluate
journeys between a start and end point. The red line shows the fastest
route before any changes you’ve made, and the blue line shows the new
route accounting for your new modal filters, one-ways, and turn
restrictions. When you see just a blue line, it means both routes are the
same – your changes had no effect on this journey.`,`The choice of route and the estimated journey time is based on simple
assumptions that drivers travel at the full speed limit, with no delays at
junctions or due to traffic. This is of course unrealistic, but there is
no openly available traffic data everywhere. Usually the fastest route
stays on main roads, which have higher speed limits, but during heavy
traffic, drivers are more likely to divert through a neighbourhood street.
You can model this situation using the slider to slow-down main road
traffic.`,"Effect on routes to one destination",`Another concern during public consultations is the effect on residents
within an LTN who drive. Previously they may have taken a shortcut through
the neighbourhood to visit the city centre, but a new filter might make
their journey slightly more inconvenient. You can use a tool to explore
the change in journey times starting from everywhere in the neighbourhood
going to one destination, designated by the orange X. Starting a journey
from most streets isn’t affected by new filters, but a few streets are
coloured red.`,`Hovering on one of the streets shows the journey before and after the
changes. You can click any of these to open in the route tool and explore
further.`,"Impact prediction",`Suppose a large volume of traffic previously took a shortcut through a
neighbourhood. After designing an LTN to address this problem, will those
drivers stick to main roads, or is there a different detour through an
adjacent neighbourhood they might try? To understand these possible
spillover effects, we need to understand the overall patterns of traffic
in the wider study area. Origin/destination datasets describe where
journeys begin and end. The LTN tool’s impact prediction mode calculates
the route each trip would take before and after your edits, and then
identifies red streets in the entire study area that may experience higher
traffic and green streets that should experience lower traffic. In the
example below, there are two LTNs, shown as grey areas, each with new
modal filters.`,["There are many assumptions and limitations with this analysis; it is ",[0,"not"],` intended to replace a proper traffic model. It is simply a convenient tool
to quickly estimate what main roads and other neighbourhoods might need attention.
The limitations include:`],`The origin/destination data for Scotland comes from the 2011
home-to-work census data. 2011 is very old, this dataset has its own
caveats, and home-to-work trips only account for a small fraction of
traffic. There are no known better open datasets to replace this.`,`By default, this tool uses the “Calculate quickly” option, which samples
only one journey between census zones, and weights the result based on
the number of trips between the zones. “Calculate more accurately” takes
longer, but simulates many journeys between zones.`,`In studies of real LTNs, counters show “traffic dissipation” over a long
period of time, in which people previously choosing to drive change
their travel behavior entirely – resulting in different destinations,
walking or cycling or taking public transit instead, driving at
different times of day, and so on. This analysis does not model any of
that.`,"Sharing your work",`All of your projects are stored in your web browser’s local storage. If
you change devices or browsers or clear your browser’s storage, then you
will not see your old projects. At any time, you can export a project to a
file from the main screen:`,`This will download a GeoJSON file. You can email this, copy to Sharepoint,
or otherwise transfer to somebody else. At the bottom of the very first
Choose Project screen, you can then load this project from its file:`,"Multiple proposals",`You may want to try a few different proposals for an LTN. Each alternate
proposal will be in its own project. From the main screen, you can quickly
copy a project and switch between projects.`,"Appendix","Changelog",["As this tool is updated, major changes will be described here. See ",[0,"Github"]," for detailed changes."],"v1, 4 June 2025 - first main release",["12 June 2025 - detecting more shortcuts (see ",[0,"details"],") and more conveniently reclassify main roads along a route (see ",[1,"details"],")"],["16 June 2025 - show a neighbourhood before any edits (see ",[0,"details"],")"],["30 June 2025 - copy and switch projects quickly (see ",[0,"details"],")"],["6 July 2025 - improve styling for pedestrianized areas (see ",[0,"details"],")"],"Credits","Table of Contents","Polygon","Click the map to add three points. Then adjust the points or add more.",[[0]," Snap boundary to roads"],"Undo",["Undo (",0,")"],[[0,"Click"]," to toggle snapping"],[0," shortcuts through ",1," (",2," mph)"],["Main road: ",0," (",1," mph)"],"Feature","LineString","INPUT","Debug","Editing tools",[[0]," Show before edits"],"Add a modal filter (hotkey 1)","Add a modal filter","Toggle one-way (hotkey 2)","Reverse directions","Restrict turns (hotkey 3)","Restrict turns","Reclassify main roads (hotkey 4)","Change main/minor roads","Undo Ctrl+Z",["Undo (",0,") Ctrl+Z"],"Redo Ctrl+Y",["Redo (",0,") Ctrl+Y"],"Add modal filter",`Modal filters restrict what kind of traffic can pass through a road
segment. Place them strategically to deter shortcuts through your
neighbourhood.`,"Change modal filter type","Add many modal filters along a line","Add along a line","Toggle one-way",`Click on a road segment to toggle its direction. This will change
the direction of traffic flow on that road.`,`To restrict certain turns, first click on the source road, then the
destination road. Traffic will no longer be able to turn from the
source road to the destination road.`,"Reclassify main roads",[[0,"Main roads"],`, drawn in grey, were classified automatically
using data from `,[1,"OpenStreetMap"],", but you can reclassify a road segment by clicking on it."],`Main roads are typically better suited to support higher levels of
traffic than neighbourhood roads.`,"Click a road to reclassify it","Toggle segment","Reclassify multiple roads by drawing a route crossing them","Mark as main along a route","Erase main classification","Erase all main roads",`Some parts of the neighbourhood aren't reachable by drivers, shown in
red`,"Map style",[[0]," Animate shortcuts",[1,"1"]],[[0]," Show entries into cells",[1,"2"]],[[0]," Road thickness depends on shortcuts",[1,"1"]],"Draw roads:","Worst shortcuts","Cell","Edits (either filter or direction)","Speed limit","Neighbourhood stats",[[0]," Click to add modal filter"],"Click to change direction","Click to designate a main road or not","Click to create a turn restriction from here","Click to delete filter","Click to delete turn restriction","Add turn restriction",[[0]," Create a turn restriction from ",0," to ",1],["Language ",[0,[0,"English"]," ",[1,"Français"]," ",[2,"Magyar"]]],[[0]," Language ",[1,[0,"English"]," ",[1,"Français"]," ",[2,"Magyar"]]],[`The Connected Neighbourhood Tool helps you design a Low-Traffic
Neighbourhood (LTN). This guide is written specifically for the Scottish
version of the tool, but most of it applies elsewhere too. In this
guide, we assume you are already familiar with the concept and purpose
of LTNs. If you have any trouble using the tool, please email the
maintainer at `,[0,"dabreegster@gmail.com"]," or ",[1,"create a Github issue"],"."],`Design your LTN with modal filters, one-way streets, turn
restrictions, and sometimes by reclassifying main roads`,["Start using the tool by going to ",[0,"https://cnt.scot/"],` and picking your study area on the map or from the list. The study areas
are defined by Local Authority Districts.`],`To design an LTN, you first need to specify its boundary. Unless you’re
working on a large circulation plan, the neighbourhood boundary will
probably be a much smaller area than the entire study area shown. You
can create multiple LTNs in one project, but you only need one to start.`,`In some cases, the boundary you want will already be shown on the map as
a coloured area. These areas are found automatically by dividing
settlements on the map by severances – main roads, railways, and bodies
of water.`,`The red points snap the boundary to roads. Sometimes near a park or body
of water without any roads, you may wish to draw the boundary in even
more detail by turning off snapping. Click any red point to turn it
blue, which you can drag anywhere you like:`,`The 1st and 2nd point form long line-like “spurs” away from the area.
Between red snapped points, the tool is trying to find the shortest
distance path along roads. Sometimes that path on both sides of a point
will use exactly the same roads, resulting in this spur. When you see
this happen, you can keep dragging points around, introducing more
points, and so on to fix the shape to match whatever you intend.`,`Population density – generally LTNs have greater benefit in denser
areas`,["Collisions – using ",[0,"stats19 data"],` about prior collisions involving pedestrians and cyclists, it may be
important to target areas with existing problems`],`Points of interest – areas with a mix of residential and commercial
land use can be important to improve walking and cycling`,[`This example neighbourhood is bounded on all sides by a grey main road,
where we assume the road is designed to handle a higher volume of
traffic. The smaller coloured areas inside the neighbourhood are `,[0,"cells"],`, showing internal connectivity for a driver. If a driver enters the
neighbourhood by the blue arrow, they are only able to reach the area
shown in blue; they can’t drive to the yellow or pink cells without
exiting back onto the main road, then re-entering the neighbourhood
somewhere else.`],`Aside from these smaller cells, this neighbourhood mainly consists of
the large blue cell. There are many points where a driver can enter and
exit this cell. Because the blue cell stretches so far, a driver can
enter from the south and drive all the way through to the north.`,`If understanding the cells as areas is confusing or inconvenient, you
can modify the map style and colour the roads by their cell instead:`,[`To design an effective LTN, you must limit the traffic cutting through
the neighbourhood. `,[0,"Shortcuts"],` show the possible routes through
a neighbourhood a driver might take when avoiding main roads. They do not
include journeys starting or ending somewhere in the neighbourhood, just
routes that pass through without stopping. These are shown in shades of red;
the white streets are dead-ends and cul-de-sacs; a driver has no reason to
go there unless their trip starts or ends there. The darkest reds show the
streets most likely to see lots of traffic cutting through. The darkest red
is along the long north/south street:`],`The tool identifies 51 different shortcuts passing through this one
street, showing the most advantageous shortcuts first – the ones that
save the driver the most time by cutting through the middle of the
neighbourhood. Most of the shortcuts are simple variations, changing the
exact entrance or exit. There are also some shortcuts involving the
western boundary:`,`For every combination, calculate the fastest driving route, using the
speed limit and length of each road. Main roads are penalised as
having half their speed limit, to simulate delays in heavy traffic
conditions.`,[`The tool assumes a driver is equally likely to enter and exit the
neighbourhood through any point, but of course this doesn’t reflect the
real traffic patterns in the larger area. Maybe the northern boundary of
this neighbourhood isn’t attractive for drivers, because there’s no
reason to drive that way. (In this case, since the neighbourhood is just
north of Aberdeen city centre and the north/south shortcut is parallel
to an A road, it `,[0,"is"],` likely a shortcut that happens in
practice.) The tool’s assumptions are necessary to make due to a lack of
detailed traffic pattern data, and because they can be calculated even
as you start to edit the neighbourhood. The shortcuts simply show what
is `,[1,"possible"],` for drivers to do, not what is likely. You may
need to apply your own local knowledge, judgment, or traffic counters to
verify a shortcut is actually a problem in practice.`],[`Now that you understand shortcuts, let’s move on to the interventions
you can propose to fix these problems. The main tool is the `,[0,"modal filter"],`, or point closure. It stops drivers from passing through a street,
while still allowing pedestrians and cyclists (and sometimes buses,
emergency vehicles, etc) through. Let’s try adding a modal filter along
the north/south shortcut:`],`The blue cell has been split into a new yellow cell, making it clear
that now the north/south shortcut is totally impossible.`,`In the scope of the tool, these all mean the same thing – a driver
cannot pass through. You can use the different types to communicate more
specific proposals. School streets are timed closures, but the tool will
model the effects of the filter during school hours. When you place a
filter on a street that currently has a bus route along it, you will
automatically get a bus gate, which uses camera enforcement and doesn’t
physically prevent vehicles from crossing. The specifics of the physical
intervention are outside the scope of this tool – depending on width
constraints, allowing adequate room for bin lorries to turn, and so on,
the physical implementation of a filter could be a pocket park,
removable bollards, concrete, etc. The LTN tool’s purpose is to focus on
the strategic planning.`,`Modal filters usually apply at one point along a street, but when you
have a four-way intersection, you can click it to toggle through two
possible diagonal filters. These allow traffic through the intersection
only for some combinations of streets.`,`You can also change the direction of traffic flow along a street. This
is helpful to retain through-traffic in one direction, but funnel it
back out to a main road. Or sometimes a shortcut is only problematic in
one direction.`,[`You may want to reclassify these main roads, and treat them like
residential streets that should not carry through-traffic. This could
make sense in the context of a larger circulation plan, a redesign to
the strategic road network in the wider area, or when the main road is a
high street with heavy foot and cycling traffic. No matter the reason,
you can mark new main roads or erase main roads using one of the tools.
In complex areas, it may be simplest to first `,[0,"Erase all main roads"]," and then ",[1,"Mark as main along a route"],`. After removing those two interior
main roads, the neighbourhood looks like one big cell:`],`As you design an LTN, you are already understanding the effects on
traffic through the area, by paying attention to cells and shortcuts.
You can also study the effects on the entire study area.`,`A common concern during public consultations is that a driving route
that previously cut through a neighbourhood will become much longer or
impossible after an LTN is created. You can use the route tool to
evaluate journeys between a start and end point. The red line shows the
fastest route before any changes you’ve made, and the blue line shows
the new route accounting for your new modal filters, one-ways, and turn
restrictions. When you see just a blue line, it means both routes are
the same – your changes had no effect on this journey.`,`The choice of route and the estimated journey time is based on simple
assumptions that drivers travel at the full speed limit, with no delays
at junctions or due to traffic. This is of course unrealistic, but there
is no openly available traffic data everywhere. Usually the fastest
route stays on main roads, which have higher speed limits, but during
heavy traffic, drivers are more likely to divert through a neighbourhood
street. You can model this situation using the slider to slow-down main
road traffic.`,`Another concern during public consultations is the effect on residents
within an LTN who drive. Previously they may have taken a shortcut
through the neighbourhood to visit the city centre, but a new filter
might make their journey slightly more inconvenient. You can use a tool
to explore the change in journey times starting from everywhere in the
neighbourhood going to one destination, designated by the orange X.
Starting a journey from most streets isn’t affected by new filters, but
a few streets are coloured red.`,`Hovering on one of the streets shows the journey before and after the
changes. You can click any of these to open in the route tool and
explore further.`,`Suppose a large volume of traffic previously took a shortcut through a
neighbourhood. After designing an LTN to address this problem, will
those drivers stick to main roads, or is there a different detour
through an adjacent neighbourhood they might try? To understand these
possible spillover effects, we need to understand the overall patterns
of traffic in the wider study area. Origin/destination datasets describe
where journeys begin and end. The LTN tool’s impact prediction mode
calculates the route each trip would take before and after your edits,
and then identifies red streets in the entire study area that may
experience higher traffic and green streets that should experience lower
traffic. In the example below, there are two LTNs, shown as grey areas,
each with new modal filters.`,`By default, this tool uses the “Calculate quickly” option, which
samples only one journey between census zones, and weights the result
based on the number of trips between the zones. “Calculate more
accurately” takes longer, but simulates many journeys between zones.`,`In studies of real LTNs, counters show “traffic dissipation” over a
long period of time, in which people previously choosing to drive
change their travel behavior entirely – resulting in different
destinations, walking or cycling or taking public transit instead,
driving at different times of day, and so on. This analysis does not
model any of that.`,`All of your projects are stored in your web browser’s local storage. If
you change devices or browsers or clear your browser’s storage, then you
will not see your old projects. At any time, you can export a project to
a file from the main screen:`,`This will download a GeoJSON file. You can email this, copy to
Sharepoint, or otherwise transfer to somebody else. At the bottom of the
very first Choose Project screen, you can then load this project from
its file:`,`You may want to try a few different proposals for an LTN. Each alternate
proposal will be in its own project. From the main screen, you can
quickly copy a project and switch between projects.`];export{e as c};
