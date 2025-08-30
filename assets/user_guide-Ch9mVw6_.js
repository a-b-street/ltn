import{av as ot,aw as st,ax as it,ay as nt,az as rt,f as lt,u as We,aA as ct,aB as je,aC as ht,q as dt,a2 as X,aD as ut,F as gt,G as Ge,ab as mt,L as N,a8 as ft,D as pt,E as o,ac as ze,v as Q,y as b,V as Fe,a3 as se,ad as Z,a0 as J,a4 as Ne,a5 as W,a6 as $,a9 as i,af as ee,M as yt,J as s,K as G,Y as Oe,ae as te,a7 as ye,I as vt,R as U,N as ae,O as oe,a1 as Ee,ar as Ye,ag as Pe,P as bt,au as wt,m as _t}from"./pico.conditional.jade.min-2xxcsZFb.js";const kt=()=>performance.now(),D={tick:a=>requestAnimationFrame(a),now:()=>kt(),tasks:new Set};function Re(){const a=D.now();D.tasks.forEach(e=>{e.c(a)||(D.tasks.delete(e),e.f())}),D.tasks.size!==0&&D.tick(Re)}function Tt(a){let e;return D.tasks.size===0&&D.tick(Re),{promise:new Promise(n=>{D.tasks.add(e={c:a,f:n})}),abort(){D.tasks.delete(e)}}}function ve(a,e){je(()=>{a.dispatchEvent(new CustomEvent(e))})}function xt(a){if(a==="float")return"cssFloat";if(a==="offset")return"cssOffset";if(a.startsWith("--"))return a;const e=a.split("-");return e.length===1?e[0]:e[0]+e.slice(1).map(n=>n[0].toUpperCase()+n.slice(1)).join("")}function qe(a){const e={},n=a.split(";");for(const c of n){const[r,y]=c.split(":");if(!r||y===void 0)break;const d=xt(r.trim());e[d]=y.trim()}return e}const At=a=>a;function Ct(a,e,n,c){var r=(a&ct)!==0,y="both",d,g=e.inert,j=e.style.overflow,m,f;function E(){return je(()=>d??=n()(e,c?.()??{},{direction:y}))}var u={is_global:r,in(){e.inert=g,ve(e,"introstart"),m=Se(e,E(),f,1,()=>{ve(e,"introend"),m?.abort(),m=d=void 0,e.style.overflow=j})},out(S){e.inert=!0,ve(e,"outrostart"),f=Se(e,E(),m,0,()=>{ve(e,"outroend"),S?.()})},stop:()=>{m?.abort(),f?.abort()}},T=ot;if((T.transitions??=[]).push(u),st){var p=r;if(!p){for(var l=T.parent;l&&(l.f&it)!==0;)for(;(l=l.parent)&&(l.f&nt)===0;);p=!l||(l.f&rt)!==0}p&&lt(()=>{We(()=>u.in())})}}function Se(a,e,n,c,r){var y=c===1;if(ht(e)){var d,g=!1;return dt(()=>{if(!g){var S=e({direction:y?"in":"out"});d=Se(a,S,n,c,r)}}),{abort:()=>{g=!0,d?.abort()},deactivate:()=>d.deactivate(),reset:()=>d.reset(),t:()=>d.t()}}if(n?.deactivate(),!e?.duration)return r(),{abort:X,deactivate:X,reset:X,t:()=>c};const{delay:j=0,css:m,tick:f,easing:E=At}=e;var u=[];if(y&&n===void 0&&(f&&f(0,1),m)){var T=qe(m(0,1));u.push(T,T)}var p=()=>1-c,l=a.animate(u,{duration:j,fill:"forwards"});return l.onfinish=()=>{l.cancel();var S=n?.t()??1-c;n?.abort();var I=c-S,w=e.duration*Math.abs(I),v=[];if(w>0){var Y=!1;if(m)for(var H=Math.ceil(w/16.666666666666668),z=0;z<=H;z+=1){var F=S+I*E(z/H),P=qe(m(F,1-F));v.push(P),Y||=P.overflow==="hidden"}Y&&(a.style.overflow="hidden"),p=()=>{var C=l.currentTime;return S+I*E(C/w)},f&&Tt(()=>{if(l.playState!=="running")return!1;var C=p();return f(C,1-C),!0})}l=a.animate(v,{duration:w,fill:"forwards"}),l.onfinish=()=>{p=()=>c,f?.(c,1-c),r()}},{abort:()=>{l&&(l.cancel(),l.effect=null,l.onfinish=X)},deactivate:()=>{r=X},reset:()=>{c===0&&f?.(1,0)},t:()=>p()}}function Nt(a,e){ut(window,["resize"],()=>je(()=>e(window[a])))}var Et=gt('<svg><path d="M3 5a1 1 0 0 1 1-1h12a1 1 0 1 1 0 2H4a1 1 0 0 1-1-1zm0 5a1 1 0 0 1 1-1h12a1 1 0 1 1 0 2H4a1 1 0 0 1-1-1zm0 5a1 1 0 0 1 1-1h12a1 1 0 1 1 0 2H4a1 1 0 0 1-1-1z"></path></svg>');function St(a,e){let n=mt(e,["$$slots","$$events","$$legacy"]);var c=Et();Ge(c,()=>({...n,viewBox:"0 0 20 20",fill:"currentColor"})),N(a,c)}function jt(a){return a<.5?4*a*a*a:.5*Math.pow(2*a-2,3)+1}function It(a){const e=typeof a=="string"&&a.match(/^\s*(-?[\d.]+)([^\s]*)\s*$/);return e?[parseFloat(e[1]),e[2]||"px"]:[a,"px"]}function Lt(a,{delay:e=0,duration:n=400,easing:c=jt,amount:r=5,opacity:y=0}={}){const d=getComputedStyle(a),g=+d.opacity,j=d.filter==="none"?"":d.filter,m=g*(1-y),[f,E]=It(r);return{delay:e,duration:n,easing:c,css:(u,T)=>`opacity: ${g-m*T}; filter: ${j} blur(${T*f}${E});`}}var Mt=(a,e)=>{a.stopPropagation(),a.preventDefault(),e(!0)},Bt=se("<button><!></button>"),Dt=se('<li role="menuitem"><!></li>'),Ht=se('<nav><!> <ol role="menu"></ol></nav>'),zt=se("<aside><!> <!></aside>");function Ft(a,e){pt(e,!0);let n=o(e,"activeHeading",15,null),c=o(e,"activeHeadingScrollOffset",3,100),r=o(e,"activeTocLi",15,null),y=o(e,"aside",15,void 0),d=o(e,"breakpoint",3,1e3),g=o(e,"desktop",15,!0),j=o(e,"flashClickedHeadingsFor",3,1500),m=o(e,"getHeadingIds",3,t=>t.id),f=o(e,"getHeadingLevels",3,t=>Number(t.nodeName[1])),E=o(e,"getHeadingTitles",3,t=>t.textContent??""),u=o(e,"headings",31,()=>ze([])),T=o(e,"headingSelector",19,()=>":is(h2, h3, h4):not(.toc-exclude)"),p=o(e,"hide",15,!1),l=o(e,"autoHide",3,!0),S=o(e,"keepActiveTocItemInView",3,!0),I=o(e,"minItems",3,0),w=o(e,"nav",15,void 0),v=o(e,"open",15,!1),Y=o(e,"openButtonLabel",19,()=>"Open table of contents"),H=o(e,"reactToKeys",19,()=>["ArrowDown","ArrowUp"," ","Enter","Escape","Tab"]),z=o(e,"scrollBehavior",19,()=>"smooth"),F=o(e,"title",19,()=>"On this page"),P=o(e,"titleTag",19,()=>"h2"),C=o(e,"tocItems",31,()=>ze([])),ie=o(e,"warnOnEmpty",3,!1),ne=o(e,"blurParams",19,()=>({duration:200})),re=o(e,"aside_style",19,()=>""),le=o(e,"aside_class",19,()=>""),ce=o(e,"nav_style",19,()=>""),he=o(e,"nav_class",19,()=>""),de=o(e,"title_element_style",19,()=>""),ue=o(e,"title_element_class",19,()=>""),ge=o(e,"ol_style",19,()=>""),be=o(e,"ol_class",19,()=>""),we=o(e,"li_style",19,()=>""),_e=o(e,"li_class",19,()=>""),Je=o(e,"open_button_style",19,()=>""),Ue=o(e,"open_button_class",19,()=>""),ke=Fe(0),Ie=Fe(!1),Ke=U(()=>u().map(f())),Ve=U(()=>Math.min(...b(Ke))||0);Q(()=>e.onOpen?.({open:v()})),Q(()=>{g(b(ke)>d())});function Xe(t){y()?.contains(t.target)||v(!1)}function Le(){if(typeof document>"u")return;const t=Array.from(document.querySelectorAll(T()));We(()=>{u(t),me(),u().length===0?(ie()&&console.warn(`svelte-toc found no headings for headingSelector='${T()}'. ${l()?"Hiding":"Showing empty"} table of contents.`),l()&&p(!0)):p()&&l()&&p(!1)})}Q(Le),Q(()=>{const t=new MutationObserver(Le);return t.observe(document.body,{childList:!0,subtree:!0,characterData:!0}),()=>t.disconnect()});function me(){let t=u().length;for(;t--;){const{top:h}=u()[t].getBoundingClientRect();if(h<c()||t===0){n(u()[t]),r(C()[t]);return}}}const Me=t=>h=>{if(h instanceof KeyboardEvent&&!["Enter"," "].includes(h.key))return;v(!1),t.scrollIntoView?.({behavior:z(),block:"start"});const L=m()&&m()(t);L&&history.replaceState({},"",`#${L}`),j()&&(t.classList.add("toc-clicked"),setTimeout(()=>t.classList.remove("toc-clicked"),j()))};function Be(t="smooth"){if(S()&&r()&&w()){const h=r()?.offsetTop-w().offsetHeight/2;w()?.scrollTo?.({top:h,behavior:t})}}Q(()=>{v()&&w()&&(me(),Be("instant"))});function Qe(t){if(!H()||!H().includes(t.key))return;const h=[...document.querySelectorAll(":hover")].at(-1),L=h&&w()?.contains(h);if(!(t.key==="Tab"&&!w()?.contains(document.activeElement)||!g()&&!v()||g()&&!L)){if(t.preventDefault(),t.key==="Escape"&&v())v(!1);else if(t.key==="Tab"&&!y()?.contains(document.activeElement))v(!1);else if(r()){if(t.key==="ArrowDown"){const M=r().nextElementSibling;M&&r(M)}if(t.key==="ArrowUp"){const M=r().previousElementSibling;M&&r(M)}n(u()[C().indexOf(r())])}r()&&[" ","Enter"].includes(t.key)&&n()?.scrollIntoView({behavior:"instant",block:"start"})}}var q=zt();Z("scroll",te,()=>{Oe(Ie,!0),me()}),Z("click",te,Xe),Z("scrollend",te,()=>{b(Ie)&&Be()}),Z("resize",te,()=>{g(b(ke)>d()),me()}),Z("keydown",te,Qe);let De;var He=G(q);{var Ze=t=>{var h=Bt();h.__click=[Mt,v];var L=G(h);{var M=_=>{var x=ae(),V=oe(x);Ee(V,()=>e.open_toc_icon),N(_,x)},K=_=>{St(_,{width:"1em"})};J(L,_=>{e.open_toc_icon?_(M):_(K,!1)})}W(()=>{i(h,"aria-label",Y()),$(h,1,ye(Ue()||null),"svelte-nl8dhp"),ee(h,Je()||null)}),N(t,h)};J(He,t=>{!v()&&!g()&&u().length>=I()&&t(Ze)})}var $e=s(He,2);{var et=t=>{var h=Ht(),L=G(h);{var M=_=>{var x=ae(),V=oe(x);{var Te=A=>{var O=ae(),R=oe(O);Ee(R,()=>e.title_snippet),N(A,O)},fe=A=>{var O=ae(),R=oe(O);bt(R,P,!1,(xe,Ae)=>{Ge(xe,()=>({class:`toc-title toc-exclude ${(ue()||"")??""}`,style:de()||null}),void 0,void 0,"svelte-nl8dhp");var pe=Ye();W(()=>Pe(pe,F())),N(Ae,pe)}),N(A,O)};J(V,A=>{e.title_snippet?A(Te):A(fe,!1)})}N(_,x)};J(L,_=>{F()&&_(M)})}var K=s(L,2);vt(K,23,u,(_,x)=>`${x}-${_.id}`,(_,x,V)=>{const Te=U(()=>f()(b(x))),fe=U(()=>b(Te)-b(Ve));var A=Dt();let O,R;var xe=U(()=>Me(b(x)));A.__click=function(...k){b(xe)?.apply(this,k)};var Ae=U(()=>Me(b(x)));A.__keydown=function(...k){b(Ae)?.apply(this,k)};var pe=G(A);{var tt=k=>{var B=ae(),Ce=oe(B);Ee(Ce,()=>e.toc_item,()=>b(x)),N(k,B)},at=k=>{var B=Ye();W(Ce=>Pe(B,Ce),[()=>E()(b(x))]),N(k,B)};J(pe,k=>{e.toc_item?k(tt):k(at,!1)})}Ne(A,(k,B)=>C(C()[B]=k,!0),k=>C()?.[k],()=>[b(V)]),W((k,B)=>{O=$(A,1,ye(_e()||null),"svelte-nl8dhp",O,k),R=ee(A,we()||null,R,B)},[()=>({active:b(x)===n()}),()=>({"margin-left":`${b(fe)??""}em`,"font-size":`${Math.max(3-b(fe)*.1,2)}ex`})]),N(_,A)}),Ne(h,_=>w(_),()=>w()),W(()=>{$(h,1,ye(he()||null),"svelte-nl8dhp"),ee(h,ce()||null),$(K,1,ye(be()||null),"svelte-nl8dhp"),ee(K,ge()||null)}),Ct(3,h,()=>Lt,ne),N(t,h)};J($e,t=>{(v()||g()&&u().length>=I())&&t(et)})}Ne(q,t=>y(t),()=>y()),W(t=>{De=$(q,1,`toc ${(le()||"")??""}`,"svelte-nl8dhp",De,t),q.hidden=p(),i(q,"aria-hidden",p()),ee(q,re()||null)},[()=>({desktop:g(),hidden:p(),mobile:!g()})]),Nt("innerWidth",t=>Oe(ke,t,!0)),N(a,q),yt()}ft(["click","keydown"]);const Ot="/ltn/assets/export_project-DnPCVa3b.webp",Yt="/ltn/assets/image01-C2DhgFIc.webp",Pt="/ltn/assets/image02-Dxw0byGy.webp",qt="/ltn/assets/image03-BkXXHkL2.webp",Wt="/ltn/assets/image04-C79hKNNL.webp",Gt="/ltn/assets/image05-CFEArA-D.webp",Rt="/ltn/assets/image06-BlNcZKlF.webp",Jt="/ltn/assets/image07-Bg9boGG1.webp",Ut="/ltn/assets/image08-DRTTYfEX.webp",Kt="/ltn/assets/image09-CcacyY1t.webp",Vt="/ltn/assets/image10-B7oszHsT.webp",Xt="/ltn/assets/image11-BvGg7YBh.webp",Qt="/ltn/assets/image12-DrxecUxv.webp",Zt="/ltn/assets/image13-Bi9xVBx2.webp",$t="/ltn/assets/image14-DVeCBqG2.webp",ea="/ltn/assets/image15-BXk_bAps.webp",ta="/ltn/assets/image16-gaRnCX_e.webp",aa="/ltn/assets/image17-DzKMjvWE.webp",oa="/ltn/assets/image18-BSvJo2WE.webp",sa="/ltn/assets/image19-DI5ywX7R.webp",ia="/ltn/assets/image20-C4lsM02Z.webp",na="/ltn/assets/image21-C_WSM11z.webp",ra="/ltn/assets/image22-C00Cd0Aw.webp",la="/ltn/assets/image23-DMPusvwR.webp",ca="/ltn/assets/image24-BPgxhWQX.webp",ha="/ltn/assets/image25-C9tPPi_h.webp",da="/ltn/assets/image26-DrS5rMFM.webp",ua="/ltn/assets/image27-COmJNFHv.webp",ga="/ltn/assets/image28-BFMHzwkd.webp",ma="/ltn/assets/image29-CTNEOjl2.webp",fa="/ltn/assets/image31-CFT3zSid.webp",pa="/ltn/assets/multiple_proposals-TMzTtEv5.webp";var ya=se(`<div class="container svelte-t533ak"><div class="contents svelte-t533ak"><h1>User guide for the Connected Neighbourhood Tool</h1> <h2 id="overview">Overview</h2> <p>The Connected Neighbourhood Tool helps you design a Low-Traffic
      Neighbourhood (LTN). This guide is written specifically for the Scottish
      version of the tool, but most of it applies elsewhere too. In this guide,
      we assume you are already familiar with the concept and purpose of LTNs.
      If you have any trouble using the tool, please email the maintainer at <a href="mailto:dabreegster@gmail.com">dabreegster@gmail.com</a> or <a href="https://github.com/a-b-street/ltn/issues" target="_blank">create a Github issue</a>.</p> <p>The overall process of using the CNT looks like this:</p> <ol><li>Choose your study area</li> <li>Add one or more neighbourhood boundaries</li> <li>Design your LTN with modal filters, one-way streets, turn restrictions,
        and sometimes by reclassifying main roads</li> <li>Explore the effects of your proposal</li> <li>Share your work with a colleague</li></ol> <h2 id="adding-a-neighbourhood">Adding a neighbourhood</h2> <p>Start using the tool by going to <a href="https://cnt.scot/">https://cnt.scot/</a> and picking your study area on the map or from the list. The study areas are
      defined by Local Authority Districts.</p> <img alt="" class="svelte-t533ak"/> <p>To design an LTN, you first need to specify its boundary. Unless you’re
      working on a large circulation plan, the neighbourhood boundary will
      probably be a much smaller area than the entire study area shown. You can
      create multiple LTNs in one project, but you only need one to start.</p> <h3 id="quick-boundaries-from-severances">Quick boundaries from severances</h3> <p>In some cases, the boundary you want will already be shown on the map as a
      coloured area. These areas are found automatically by dividing settlements
      on the map by severances – main roads, railways, and bodies of water.</p> <img alt="" class="svelte-t533ak"/> <p>After clicking one area, you can keep clicking adjacent areas to extend
      the boundary, in case the first boundary is too small.</p> <img alt="" class="svelte-t533ak"/> <h3 id="drawing-manually">Drawing manually</h3> <p>Alternatively, you can draw an area in more detail by picking at least
      three points on the map.</p> <img alt="" class="svelte-t533ak"/> <p>You can drag any of the red or grey points to adjust the boundary. Any
      point you drag becomes a red waypoint:</p> <img alt="" class="svelte-t533ak"/> <p>The red points snap the boundary to roads. Sometimes near a park or body
      of water without any roads, you may wish to draw the boundary in even more
      detail by turning off snapping. Click any red point to turn it blue, which
      you can drag anywhere you like:</p> <img alt="" class="svelte-t533ak"/> <p>When you draw a boundary manually, sometimes the resulting area doesn’t
      have a valid shape:</p> <img alt="" class="svelte-t533ak"/> <p>The 1st and 2nd point form long line-like “spurs” away from the area.
      Between red snapped points, the tool is trying to find the shortest
      distance path along roads. Sometimes that path on both sides of a point
      will use exactly the same roads, resulting in this spur. When you see this
      happen, you can keep dragging points around, introducing more points, and
      so on to fix the shape to match whatever you intend.</p> <h3 id="prioritisation-metrics">Prioritisation metrics</h3> <p>This feature is currently available in Scotland only.</p> <p><a href="https://content.tfl.gov.uk/lsp-app-six-b-strategic-neighbourhoods-analysis-v1.pdf">Transport for London’s Strategic Neighbourhood Analysis</a> describes an approach for prioritising LTNs by different metrics. The CNT
      exposes some of these metrics for the areas:</p> <ul><li>Population density – generally LTNs have greater benefit in denser areas</li> <li>Collisions – using <a href="https://www.gov.uk/government/collections/road-accidents-and-safety-statistics">stats19 data</a> about prior collisions involving pedestrians and cyclists, it may be important
        to target areas with existing problems</li> <li>SIMD (Scottish Index of Multiple Deprivation) – depending on local
        priorities, areas with higher deprivation may be important</li> <li>Points of interest – areas with a mix of residential and commercial land
        use can be important to improve walking and cycling</li> <li>Car ownership – residents in areas with low car ownership are not
        benefitting from through-traffic</li></ul> <p>Depending on local priorities, you may want to use some combination of
      these metrics to decide where to prioritise creating an LTN. You can
      colour the areas by any of these metrics:</p> <img alt="" class="svelte-t533ak"/> <p>As you select or draw a boundary, all of these metrics are evaluated
      against your area:</p> <img alt="" class="svelte-t533ak"/> <h2 id="designing-an-ltn">Designing an LTN</h2> <p>After specifying a neighbourhood boundary, you are in the main editing
      mode. There are four editing controls available, but first you need to
      understand the cells and shortcuts shown on the map.</p> <img alt="" class="svelte-t533ak"/> <h3 id="understanding-the-map-cells">Understanding the map: cells</h3> <p>This example neighbourhood is bounded on all sides by a grey main road,
      where we assume the road is designed to handle a higher volume of traffic.
      The smaller coloured areas inside the neighbourhood are <strong>cells</strong>, showing internal connectivity for a driver. If a driver enters the
      neighbourhood by the blue arrow, they are only able to reach the area
      shown in blue; they can’t drive to the yellow or pink cells without
      exiting back onto the main road, then re-entering the neighbourhood
      somewhere else.</p> <img alt="" class="svelte-t533ak"/> <p>Another example is shown below. The orange cell is effectively a small
      cul-de-sac; a driver won’t enter unless their journey starts or ends
      there. They can’t access the larger blue cell.</p> <img alt="" class="svelte-t533ak"/> <p>Aside from these smaller cells, this neighbourhood mainly consists of the
      large blue cell. There are many points where a driver can enter and exit
      this cell. Because the blue cell stretches so far, a driver can enter from
      the south and drive all the way through to the north.</p> <img alt="" class="svelte-t533ak"/> <p>If understanding the cells as areas is confusing or inconvenient, you can
      modify the map style and colour the roads by their cell instead:</p> <img alt="" class="svelte-t533ak"/> <h3 id="understanding-the-map-shortcuts">Understanding the map: shortcuts</h3> <p>To design an effective LTN, you must limit the traffic cutting through the
      neighbourhood. <strong>Shortcuts</strong> show the possible routes through
      a neighbourhood a driver might take when avoiding main roads. They do not include
      journeys starting or ending somewhere in the neighbourhood, just routes that
      pass through without stopping. These are shown in shades of red; the white
      streets are dead-ends and cul-de-sacs; a driver has no reason to go there unless
      their trip starts or ends there. The darkest reds show the streets most likely
      to see lots of traffic cutting through. The darkest red is along the long north/south
      street:</p> <img alt="" class="svelte-t533ak"/> <p>To understand why, you can use the Shortcuts tool at the top. If we
      inspect this street, we see one example shortcut from north to south:</p> <img alt="" class="svelte-t533ak"/> <p>The tool identifies 51 different shortcuts passing through this one
      street, showing the most advantageous shortcuts first – the ones that save
      the driver the most time by cutting through the middle of the
      neighbourhood. Most of the shortcuts are simple variations, changing the
      exact entrance or exit. There are also some shortcuts involving the
      western boundary:</p> <img alt="" class="svelte-t533ak"/> <p>The tool counts these shortcuts in a simple way:</p> <ol><li>Find all entrances and exits onto streets from the main roads</li> <li>For every combination, calculate the fastest driving route, using the
        speed limit and length of each road. Main roads are penalised as having
        half their speed limit, to simulate delays in heavy traffic conditions.</li> <li>Any route that crosses a main road is discarded</li> <li>Count the number of routes crossing each street segment</li></ol> <p>The tool assumes a driver is equally likely to enter and exit the
      neighbourhood through any point, but of course this doesn’t reflect the
      real traffic patterns in the larger area. Maybe the northern boundary of
      this neighbourhood isn’t attractive for drivers, because there’s no reason
      to drive that way. (In this case, since the neighbourhood is just north of
      Aberdeen city centre and the north/south shortcut is parallel to an A
      road, it <strong>is</strong> likely a shortcut that happens in practice.)
      The tool’s assumptions are necessary to make due to a lack of detailed
      traffic pattern data, and because they can be calculated even as you start
      to edit the neighbourhood. The shortcuts simply show what is <strong>possible</strong> for drivers to do, not what is likely. You may need
      to apply your own local knowledge, judgment, or traffic counters to verify
      a shortcut is actually a problem in practice.</p> <h3 id="editing-modal-filters">Editing: modal filters</h3> <p>Now that you understand shortcuts, let’s move on to the interventions you
      can propose to fix these problems. The main tool is the <strong>modal filter</strong>, or point closure. It stops drivers from passing through a street, while
      still allowing pedestrians and cyclists (and sometimes buses, emergency
      vehicles, etc) through. Let’s try adding a modal filter along the
      north/south shortcut:</p> <img alt="" class="svelte-t533ak"/> <p>Immediately after you click to add the filter, you’ll see the red
      shortcuts jump to the right, zig-zagging to avoid the new filter. If you
      add a second filter there, you’ll see a big change:</p> <img alt="" class="svelte-t533ak"/> <p>The blue cell has been split into a new yellow cell, making it clear that
      now the north/south shortcut is totally impossible.</p> <p>You may have noticed the modal filter icons on the map are different.
      There are four types you can choose from:</p> <img alt="" class="svelte-t533ak"/> <p>In the scope of the tool, these all mean the same thing – a driver cannot
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
      planning.</p> <h3 id="editing-diagonal-modal-filters">Editing: diagonal modal filters</h3> <p>Modal filters usually apply at one point along a street, but when you have
      a four-way intersection, you can click it to toggle through two possible
      diagonal filters. These allow traffic through the intersection only for
      some combinations of streets.</p> <img alt="" class="svelte-t533ak"/> <h3 id="editing-one-way-streets">Editing: one-way streets</h3> <p>You can also change the direction of traffic flow along a street. This is
      helpful to retain through-traffic in one direction, but funnel it back out
      to a main road. Or sometimes a shortcut is only problematic in one
      direction.</p> <img alt="" class="svelte-t533ak"/> <p>You cannot create new cells only by introducing one-way streets, but you
      can influence shortcuts.</p> <h3 id="editing-turn-restrictions">Editing: turn restrictions</h3> <p>You can restrict some turns through an intersection without outright
      preventing all movement. This may be useful to prevent unprotected turns
      to or from a main road when there is no room for a turning lane.</p> <img alt="" class="svelte-t533ak"/> <p>Note that existing turn restrictions are automatically added from
      OpenStreetMap data. There are some complex situations near dual
      carriageways that may not be detected correctly; please contact the team
      to report this problem if you encounter one.</p> <h3 id="editing-main-road-classification">Editing: main road classification</h3> <p>When you initially create a neighbourhood from its boundary, some roads
      count as <strong>main roads</strong>, shown in grey. The initial
      classification is taken from OpenStreetMap data. Main roads are intended
      to handle through-traffic, and so the tool does not calculate shortcuts
      along main roads, and the cells are determined by connections to main
      roads. In the example below, there are main roads surrounding the
      perimeter of the neighbourhood, which is typical, but there are also two
      north/south main roads in the middle, causing there to be cells on each
      side.</p> <img alt="" class="svelte-t533ak"/> <p>You may want to reclassify these main roads, and treat them like
      residential streets that should not carry through-traffic. This could make
      sense in the context of a larger circulation plan, a redesign to the
      strategic road network in the wider area, or when the main road is a high
      street with heavy foot and cycling traffic. No matter the reason, you can
      mark new main roads or erase main roads using one of the tools. In complex
      areas, it may be simplest to first <i>Erase all main roads</i> and then <i>Mark as main along a route</i>. After removing those two interior main
      roads, the neighbourhood looks like one big cell:</p> <img alt="" class="svelte-t533ak"/> <p>You can now make other edits and see the effects on cells and shortcuts
      through the entire area.</p> <h2 id="exploring-effects">Exploring effects</h2> <p>As you design an LTN, you are already understanding the effects on traffic
      through the area, by paying attention to cells and shortcuts. You can also
      study the effects on the entire study area.</p> <h3 id="effect-on-one-journey">Effect on one journey</h3> <p>A common concern during public consultations is that a driving route that
      previously cut through a neighbourhood will become much longer or
      impossible after an LTN is created. You can use the route tool to evaluate
      journeys between a start and end point. The red line shows the fastest
      route before any changes you’ve made, and the blue line shows the new
      route accounting for your new modal filters, one-ways, and turn
      restrictions. When you see just a blue line, it means both routes are the
      same – your changes had no effect on this journey.</p> <img alt="" class="svelte-t533ak"/> <p>The choice of route and the estimated journey time is based on simple
      assumptions that drivers travel at the full speed limit, with no delays at
      junctions or due to traffic. This is of course unrealistic, but there is
      no openly available traffic data everywhere. Usually the fastest route
      stays on main roads, which have higher speed limits, but during heavy
      traffic, drivers are more likely to divert through a neighbourhood street.
      You can model this situation using the slider to slow-down main road
      traffic.</p> <h3 id="effect-on-routes-to-one-destination">Effect on routes to one destination</h3> <p>Another concern during public consultations is the effect on residents
      within an LTN who drive. Previously they may have taken a shortcut through
      the neighbourhood to visit the city centre, but a new filter might make
      their journey slightly more inconvenient. You can use a tool to explore
      the change in journey times starting from everywhere in the neighbourhood
      going to one destination, designated by the orange X. Starting a journey
      from most streets isn’t affected by new filters, but a few streets are
      coloured red.</p> <img alt="" class="svelte-t533ak"/> <p>Hovering on one of the streets shows the journey before and after the
      changes. You can click any of these to open in the route tool and explore
      further.</p> <img alt="" class="svelte-t533ak"/> <h3 id="impact-prediction">Impact prediction</h3> <p>Suppose a large volume of traffic previously took a shortcut through a
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
      modal filters.</p> <img alt="" class="svelte-t533ak"/> <p>There are many assumptions and limitations with this analysis; it is <strong>not</strong> intended to replace a proper traffic model. It is simply a convenient tool
      to quickly estimate what main roads and other neighbourhoods might need attention.
      The limitations include:</p> <ul><li>The origin/destination data for Scotland comes from the 2011
        home-to-work census data. 2011 is very old, this dataset has its own
        caveats, and home-to-work trips only account for a small fraction of
        traffic. There are no known better open datasets to replace this.</li> <li>By default, this tool uses the “Calculate quickly” option, which samples
        only one journey between census zones, and weights the result based on
        the number of trips between the zones. “Calculate more accurately” takes
        longer, but simulates many journeys between zones.</li> <li>In studies of real LTNs, counters show “traffic dissipation” over a long
        period of time, in which people previously choosing to drive change
        their travel behavior entirely – resulting in different destinations,
        walking or cycling or taking public transit instead, driving at
        different times of day, and so on. This analysis does not model any of
        that.</li></ul> <h2 id="sharing-your-work">Sharing your work</h2> <p>All of your projects are stored in your web browser’s local storage. If
      you change devices or browsers or clear your browser’s storage, then you
      will not see your old projects. At any time, you can export a project to a
      file from the main screen:</p> <img alt="" class="svelte-t533ak"/> <p>This will download a GeoJSON file. You can email this, copy to Sharepoint,
      or otherwise transfer to somebody else. At the bottom of the very first
      Choose Project screen, you can then load this project from its file:</p> <img alt="" class="svelte-t533ak"/> <h3 id="multiple-proposals">Multiple proposals</h3> <p>You may want to try a few different proposals for an LTN. Each alternate
      proposal will be in its own project. From the main screen, you can quickly
      copy a project and switch between projects.</p> <img alt="" class="svelte-t533ak"/> <h2 id="appendix">Appendix</h2> <h3 id="changelog">Changelog</h3> <p>As this tool is updated, major changes will be described here. See <a href="https://github.com/a-b-street/ltn/commits/main/" target="_blank">Github</a> for detailed changes.</p> <ul><li>v1, 4 June 2025 - first main release</li> <li>12 June 2025 - detecting more shortcuts (see <a href="https://github.com/a-b-street/ltn/pull/381" target="_blank">details</a>) and more conveniently reclassify main roads along a route (see <a href="https://github.com/a-b-street/ltn/pull/382" target="_blank">details</a>)</li> <li>16 June 2025 - show a neighbourhood before any edits (see <a href="https://github.com/a-b-street/ltn/pull/379" target="_blank">details</a>)</li> <li>30 June 2025 - copy and switch projects quickly (see <a href="https://github.com/a-b-street/ltn/pull/394" target="_blank">details</a>)</li> <li>6 July 2025 - improve styling for pedestrianized areas (see <a href="https://github.com/a-b-street/ltn/pull/397" target="_blank">details</a>)</li></ul> <h3 id="credits">Credits</h3> <!></div> <div class="toc svelte-t533ak"><!></div></div>`);function va(a){var e=ya(),n=G(e),c=s(G(n),14),r=s(c,8),y=s(r,4),d=s(y,6),g=s(d,4),j=s(g,4),m=s(j,4),f=s(m,14),E=s(f,4),u=s(E,6),T=s(u,6),p=s(T,4),l=s(p,4),S=s(l,4),I=s(S,6),w=s(I,4),v=s(w,4),Y=s(v,12),H=s(Y,4),z=s(H,6),F=s(z,8),P=s(F,6),C=s(P,8),ie=s(C,8),ne=s(ie,4),re=s(ne,12),le=s(re,8),ce=s(le,4),he=s(ce,6),de=s(he,10),ue=s(de,4),ge=s(ue,6),be=s(ge,12);wt(be);var we=s(n,2),_e=G(we);Ft(_e,{title:"Table of Contents"}),W(()=>{i(c,"src",Yt),i(r,"src",Pt),i(y,"src",qt),i(d,"src",Wt),i(g,"src",Gt),i(j,"src",Rt),i(m,"src",Jt),i(f,"src",Ut),i(E,"src",Kt),i(u,"src",Vt),i(T,"src",Xt),i(p,"src",Qt),i(l,"src",Zt),i(S,"src",$t),i(I,"src",ea),i(w,"src",ta),i(v,"src",aa),i(Y,"src",oa),i(H,"src",sa),i(z,"src",ia),i(F,"src",na),i(P,"src",ra),i(C,"src",la),i(ie,"src",ca),i(ne,"src",ha),i(re,"src",da),i(le,"src",ua),i(ce,"src",ga),i(he,"src",ma),i(de,"src",Ot),i(ue,"src",fa),i(ge,"src",pa)}),N(a,e)}_t(va,{target:document.getElementById("app")});
