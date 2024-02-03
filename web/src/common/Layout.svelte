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

  .left {
    width: 25%;
    height: 100vh;
    overflow: auto;
    padding: 8px;
  }
  .main {
    width: 75%;
  }
</style>
