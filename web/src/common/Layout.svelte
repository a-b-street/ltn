<div class="top">
  <slot name="top" />
</div>
<div class="left">
  <slot name="left" />
</div>
<div class="main">
  <slot name="main" />
</div>

<style>
  div {
    float: left;
  }

  .top {
    width: 100%;
    height: 10vh;
  }

  .left {
    width: 25%;
    height: 90vh;
    overflow: auto;
    padding: 8px;
  }

  .main {
    width: 75%;
    height: 90vh;
  }
</style>
