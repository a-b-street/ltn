<script lang="ts">
  import "@picocss/pico/css/pico.conditional.jade.min.css";
  import Toc from "svelte-toc";
  import About from "../About.svelte";
  import image01 from "./image01.webp?url";
  import image02 from "./image02.webp?url";
  import image03 from "./image03.webp?url";
  import image04 from "./image04.webp?url";
  import image05 from "./image05.webp?url";
  import image06 from "./image06.webp?url";
  import image07 from "./image07.webp?url";
  import image08 from "./image08.webp?url";
  import image09 from "./image09.webp?url";
  import image10 from "./image10.webp?url";
  import image11 from "./image11.webp?url";
  import image12 from "./image12.webp?url";
  import image13 from "./image13.webp?url";
  import image14 from "./image14.webp?url";
  import image15 from "./image15.webp?url";
  import image16 from "./image16.webp?url";
  import image17 from "./image17.webp?url";
  import image18 from "./image18.webp?url";
  import image19 from "./image19.webp?url";
  import image20 from "./image20.webp?url";
  import image21 from "./image21.webp?url";
  import image22 from "./image22.webp?url";
  import image23 from "./image23.webp?url";
  import image24 from "./image24.webp?url";
  import image25 from "./image25.webp?url";
  import image26 from "./image26.webp?url";
  import image27 from "./image27.webp?url";
  import image28 from "./image28.webp?url";
  import image29 from "./image29.webp?url";
  import image30 from "./image30.webp?url";
  import image31 from "./image31.webp?url";
</script>

<div class="container">
  <div class="contents">
    <h1>User guide for the Low-Traffic Neighbourhood Tool</h1>

    <h2 id="overview">Overview</h2>
    <p>
      The Connected Neighbourhood Tool helps you design a Low-Traffic
      Neighbourhood (LTN). This guide is written specifically for the Scottish
      version of the tool, but most of it applies elsewhere too. In this guide,
      we assume you are already familiar with the concept and purpose of LTNs.
      If you have any trouble using the tool, please email the maintainer at <a
        href="mailto:dabreegster@gmail.com">dabreegster@gmail.com</a
      >.
    </p>
    <p>The overall process of using the CNT looks like this:</p>
    <ol>
      <li>Choose your study area</li>
      <li>Add one or more neighbourhood boundaries</li>
      <li>
        Design your LTN with modal filters, one-way streets, turn restrictions,
        and sometimes by reclassifying main roads
      </li>
      <li>Explore the effects of your proposal</li>
      <li>Share your work with a colleague</li>
    </ol>

    <h2 id="adding-a-neighbourhood">Adding a neighbourhood</h2>
    <p>
      Start using the tool by going to <a href="https://cnt.scot/"
        >https://cnt.scot/</a
      >
      and picking your study area on the map or from the list. The study areas are
      defined by Local Authority Districts.
    </p>
    <img alt="" src={image01} />
    <p>
      To design an LTN, you first need to specify its boundary. Unless you’re
      working on a large circulation plan, the neighbourhood boundary will
      probably be a much smaller area than the entire study area shown. You can
      create multiple LTNs in one project, but you only need one to start.
    </p>

    <h3 id="quick-boundaries-from-severances">
      Quick boundaries from severances
    </h3>
    <p>
      In some cases, the boundary you want will already be shown on the map as a
      coloured area. These areas are found automatically by dividing settlements
      on the map by severances – main roads, railways, and bodies of water.
    </p>
    <img alt="" src={image02} />
    <p>
      After clicking one area, you can keep clicking adjacent areas to extend
      the boundary, in case the first boundary is too small.
    </p>
    <img alt="" src={image03} />

    <h3 id="drawing-manually">Drawing manually</h3>
    <p>
      Alternatively, you can draw an area in more detail by picking at least
      three points on the map.
    </p>
    <img alt="" src={image04} />
    <p>
      You can drag any of the red or grey points to adjust the boundary. Any
      point you drag becomes a red waypoint:
    </p>
    <img alt="" src={image05} />
    <p>
      The red points snap the boundary to roads. Sometimes near a park or body
      of water without any roads, you may wish to draw the boundary in even more
      detail by turning off snapping. Click any red point to turn it blue, which
      you can drag anywhere you like:
    </p>
    <img alt="" src={image06} />
    <p>
      When you draw a boundary manually, sometimes the resulting area doesn’t
      have a valid shape:
    </p>
    <img alt="" src={image07} />
    <p>
      The 1st and 2nd point form long line-like “spurs” away from the area.
      Between red snapped points, the tool is trying to find the shortest
      distance path along roads. Sometimes that path on both sides of a point
      will use exactly the same roads, resulting in this spur. When you see this
      happen, you can keep dragging points around, introducing more points, and
      so on to fix the shape to match whatever you intend.
    </p>

    <h3 id="prioritisation-metrics">Prioritisation metrics</h3>
    <p>This feature is currently available in Scotland only.</p>
    <p>
      <a
        href="https://content.tfl.gov.uk/lsp-app-six-b-strategic-neighbourhoods-analysis-v1.pdf"
        >Transport for London’s Strategic Neighbourhood Analysis</a
      > describes an approach for prioritising LTNs by different metrics. The CNT
      exposes some of these metrics for the areas:
    </p>
    <ul>
      <li>
        Population density – generally LTNs have greater benefit in denser areas
      </li>
      <li>
        Collisions – using <a
          href="https://www.gov.uk/government/collections/road-accidents-and-safety-statistics"
          >stats19 data</a
        > about prior collisions involving pedestrians and cyclists, it may be important
        to target areas with existing problems
      </li>
      <li>
        SIMD (Scottish Index of Multiple Deprivation) – depending on local
        priorities, areas with higher deprivation may be important
      </li>
      <li>
        Points of interest – areas with a mix of residential and commercial land
        use can be important to improve walking and cycling
      </li>
      <li>
        Car ownership – residents in areas with low car ownership are not
        benefitting from through-traffic
      </li>
    </ul>
    <p>
      Depending on local priorities, you may want to use some combination of
      these metrics to decide where to prioritise creating an LTN. You can
      colour the areas by any of these metrics:
    </p>
    <img alt="" src={image08} />
    <p>
      As you select or draw a boundary, all of these metrics are evaluated
      against your area:
    </p>
    <img alt="" src={image09} />

    <h2 id="designing-an-ltn">Designing an LTN</h2>
    <p>
      After specifying a neighbourhood boundary, you are in the main editing
      mode. There are four editing controls available, but first you need to
      understand the cells and shortcuts shown on the map.
    </p>
    <img alt="" src={image10} />

    <h3 id="understanding-the-map-cells">Understanding the map: cells</h3>
    <p>
      This example neighbourhood is bounded on all sides by a grey main road,
      where we assume the road is designed to handle a higher volume of traffic.
      The smaller coloured areas inside the neighbourhood are <strong
        >cells</strong
      >, showing internal connectivity for a driver. If a driver enters the
      neighbourhood by the blue arrow, they are only able to reach the area
      shown in blue; they can’t drive to the yellow or pink cells without
      exiting back onto the main road, then re-entering the neighbourhood
      somewhere else.
    </p>
    <img alt="" src={image11} />
    <p>
      Another example is shown below. The orange cell is effectively a small
      cul-de-sac; a driver won’t enter unless their journey starts or ends
      there. They can’t access the larger blue cell.
    </p>
    <img alt="" src={image12} />
    <p>
      Aside from these smaller cells, this neighbourhood mainly consists of the
      large blue cell. There are many points where a driver can enter and exit
      this cell. Because the blue cell stretches so far, a driver can enter from
      the south and drive all the way through to the north.
    </p>
    <img alt="" src={image13} />
    <p>
      If understanding the cells as areas is confusing or inconvenient, you can
      modify the map style and colour the roads by their cell instead:
    </p>
    <img alt="" src={image14} />

    <h3 id="understanding-the-map-shortcuts">
      Understanding the map: shortcuts
    </h3>
    <p>
      To design an effective LTN, you must limit the traffic cutting through the
      neighbourhood. <strong>Shortcuts</strong> show the possible routes through
      a neighbourhood a driver might take when avoiding main roads. They do not include
      journeys starting or ending somewhere in the neighbourhood, just routes that
      pass through without stopping. These are shown in shades of red; the white
      streets are dead-ends and cul-de-sacs; a driver has no reason to go there unless
      their trip starts or ends there. The darkest reds show the streets most likely
      to see lots of traffic cutting through. The darkest red is along the long north/south
      street:
    </p>
    <img alt="" src={image15} />
    <p>
      To understand why, you can use the Shortcuts tool at the top. If we
      inspect this street, we see one example shortcut from north to south:
    </p>
    <img alt="" src={image16} />
    <p>
      The tool identifies 51 different shortcuts passing through this one
      street, showing the most advantageous shortcuts first – the ones that save
      the driver the most time by cutting through the middle of the
      neighbourhood. Most of the shortcuts are simple variations, changing the
      exact entrance or exit. There are also some shortcuts involving the
      western boundary:
    </p>
    <img alt="" src={image17} />
    <p>The tool counts these shortcuts in a simple way:</p>
    <ol>
      <li>Find all entrances and exits onto streets from the main roads</li>
      <li>
        For every combination, calculate the shortest driving route, restricted
        to not cross a main road
      </li>
      <li>Count the number of routes crossing each street segment</li>
    </ol>
    <p>
      The tool assumes a driver is equally likely to enter and exit the
      neighbourhood through any point, but of course this doesn’t reflect the
      real traffic patterns in the larger area. Maybe the northern boundary of
      this neighbourhood isn’t attractive for drivers, because there’s no reason
      to drive that way. (In this case, since the neighbourhood is just north of
      Aberdeen city centre and the north/south shortcut is parallel to an A
      road, it <strong>is</strong> likely a shortcut that happens in practice.)
      The tool’s assumptions are necessary to make due to a lack of detailed
      traffic pattern data, and because they can be calculated even as you start
      to edit the neighbourhood. The shortcuts simply show what is
      <strong>possible</strong> for drivers to do, not what is likely. You may need
      to apply your own local knowledge, judgment, or traffic counters to verify
      a shortcut is actually a problem in practice.
    </p>

    <h3 id="editing-modal-filters">Editing: modal filters</h3>
    <p>
      Now that you understand shortcuts, let’s move on to the interventions you
      can propose to fix these problems. The main tool is the <strong
        >modal filter</strong
      >, or point closure. It stops drivers from passing through a street, while
      still allowing pedestrians and cyclists (and sometimes buses, emergency
      vehicles, etc) through. Let’s try adding a modal filter along the
      north/south shortcut:
    </p>
    <img alt="" src={image18} />
    <p>
      Immediately after you click to add the filter, you’ll see the red
      shortcuts jump to the right, zig-zagging to avoid the new filter. If you
      add a second filter there, you’ll see a big change:
    </p>
    <img alt="" src={image19} />
    <p>
      The blue cell has been split into a new yellow cell, making it clear that
      now the north/south shortcut is totally impossible.
    </p>
    <p>
      You may have noticed the modal filter icons on the map are different.
      There are four types you can choose from:
    </p>
    <img alt="" src={image20} />
    <p>
      In the scope of the tool, these all mean the same thing – a driver cannot
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
      planning.
    </p>

    <h3 id="editing-diagonal-modal-filters">Editing: diagonal modal filters</h3>
    <p>
      Modal filters usually apply at one point along a street, but when you have
      a four-way intersection, you can click it to toggle through two possible
      diagonal filters. These allow traffic through the intersection only for
      some combinations of streets.
    </p>
    <img alt="" src={image21} />

    <h3 id="editing-one-way-streets">Editing: one-way streets</h3>
    <p>
      You can also change the direction of traffic flow along a street. This is
      helpful to retain through-traffic in one direction, but funnel it back out
      to a main road. Or sometimes a shortcut is only problematic in one
      direction.
    </p>
    <img alt="" src={image22} />
    <p>
      You cannot create new cells only by introducing one-way streets, but you
      can influence shortcuts.
    </p>

    <h3 id="editing-turn-restrictions">Editing: turn restrictions</h3>
    <p>
      You can restrict some turns through an intersection without outright
      preventing all movement. This may be useful to prevent unprotected turns
      to or from a main road when there is no room for a turning lane.
    </p>
    <img alt="" src={image23} />
    <p>
      Note that existing turn restrictions are automatically added from
      OpenStreetMap data. There are some complex situations near dual
      carriageways that may not be detected correctly; please contact the team
      to report this problem if you encounter one.
    </p>

    <h3 id="editing-main-road-classification">
      Editing: main road classification
    </h3>
    <p>
      When you initially create a neighbourhood from its boundary, some roads
      count as <strong>main roads</strong>, shown in grey. The initial
      classification is taken from OpenStreetMap data. Main roads are intended
      to handle through-traffic, and so the tool does not calculate shortcuts
      along main roads, and the cells are determined by connections to main
      roads. In the example below, there are main roads surrounding the
      perimeter of the neighbourhood, which is typical, but there are also two
      north/south main roads in the middle, causing there to be cells on each
      side.
    </p>
    <img alt="" src={image24} />
    <p>
      You may want to reclassify these main roads, and treat them like
      residential streets that should not carry through-traffic. This could make
      sense in the context of a larger circulation plan, a redesign to the
      strategic road network in the wider area, or when the main road is a high
      street with heavy foot and cycling traffic. No matter the reason, you can
      mark new main roads or erase main roads using one of the tools. After
      removing those two interior main roads, the neighbourhood looks like one
      big cell:
    </p>
    <img alt="" src={image25} />
    <p>
      You can now make other edits and see the effects on cells and shortcuts
      through the entire area.
    </p>

    <h2 id="exploring-effects">Exploring effects</h2>
    <p>
      As you design an LTN, you are already understanding the effects on traffic
      through the area, by paying attention to cells and shortcuts. You can also
      study the effects on the entire study area.
    </p>

    <h3 id="effect-on-one-journey">Effect on one journey</h3>
    <p>
      A common concern during public consultations is that a driving route that
      previously cut through a neighbourhood will become much longer or
      impossible after an LTN is created. You can use the route tool to evaluate
      journeys between a start and end point. The red line shows the fastest
      route before any changes you’ve made, and the blue line shows the new
      route accounting for your new modal filters, one-ways, and turn
      restrictions. When you see just a blue line, it means both routes are the
      same – your changes had no effect on this journey.
    </p>
    <img alt="" src={image26} />
    <p>
      The choice of route and the estimated journey time is based on simple
      assumptions that drivers travel at the full speed limit, with no delays at
      junctions or due to traffic. This is of course unrealistic, but there is
      no openly available traffic data everywhere. Usually the fastest route
      stays on main roads, which have higher speed limits, but during heavy
      traffic, drivers are more likely to divert through a neighbourhood street.
      You can model this situation using the slider to slow-down main road
      traffic.
    </p>

    <h3 id="effect-on-routes-to-one-destination">
      Effect on routes to one destination
    </h3>
    <p>
      Another concern during public consultations is the effect on residents
      within an LTN who drive. Previously they may have taken a shortcut through
      the neighbourhood to visit the city centre, but a new filter might make
      their journey slightly more inconvenient. You can use a tool to explore
      the change in journey times starting from everywhere in the neighbourhood
      going to one destination, designated by the orange X. Starting a journey
      from most streets isn’t affected by new filters, but a few streets are
      coloured red.
    </p>
    <img alt="" src={image27} />
    <p>
      Hovering on one of the streets shows the journey before and after the
      changes. You can click any of these to open in the route tool and explore
      further.
    </p>
    <img alt="" src={image28} />

    <h3 id="impact-prediction">Impact prediction</h3>
    <p>
      Suppose a large volume of traffic previously took a shortcut through a
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
      modal filters.
    </p>
    <img alt="" src={image29} />
    <p>
      There are many assumptions and limitations with this analysis; it is <strong
        >not</strong
      > intended to replace a proper traffic model. It is simply a convenient tool
      to quickly estimate what main roads and other neighbourhoods might need attention.
      The limitations include:
    </p>
    <ul>
      <li>
        The origin/destination data for Scotland comes from the 2011
        home-to-work census data. 2011 is very old, this dataset has its own
        caveats, and home-to-work trips only account for a small fraction of
        traffic. There are no known better open datasets to replace this.
      </li>
      <li>
        By default, this tool uses the “Calculate quickly” option, which samples
        only one journey between census zones, and weights the result based on
        the number of trips between the zones. “Calculate more accurately” takes
        longer, but simulates many journeys between zones.
      </li>
      <li>
        In studies of real LTNs, counters show “traffic dissipation” over a long
        period of time, in which people previously choosing to drive change
        their travel behavior entirely – resulting in different destinations,
        walking or cycling or taking public transit instead, driving at
        different times of day, and so on. This analysis does not model any of
        that.
      </li>
    </ul>

    <h2 id="sharing-your-work">Sharing your work</h2>
    <p>
      All of your projects are stored in your web browser’s local storage. If
      you change devices or browsers or clear your browser’s storage, then you
      will not see your old projects. At any time, you can export a project to a
      file from the main screen:
    </p>
    <img alt="" src={image30} />
    <p>
      This will download a GeoJSON file. You can email this, copy to Sharepoint,
      or otherwise transfer to somebody else. At the bottom of the very first
      Choose Project screen, you can then load this project from its file:
    </p>
    <img alt="" src={image31} />

    <h2 id="appendix">Appendix</h2>
    <h3 id="changelog">Changelog</h3>
    <p>As this tool is updated, changes will be described here:</p>
    <ul>
      <li>v1, XX June 2025 - first main release</li>
    </ul>

    <h3 id="credits">Credits</h3>
    <About />
  </div>
  <div class="toc">
    <Toc title="Table of Contents" />
  </div>
</div>

<style>
  .container {
    display: flex;
  }
  .contents {
    width: 80%;
    padding: 16px;
  }
  .toc {
    width: 20%;
  }

  img {
    width: 80%;
    display: block;
    margin: auto;
  }
</style>
