import {
  transformRequest,
  fetchData,
  mapInteractions,
  setupFilters,
} from "./utils.js";

const map = new maplibregl.Map({
  container: "map",
  style:
    "https://raw.githubusercontent.com/OrdnanceSurvey/OS-Vector-Tile-API-Stylesheets/main/OS_VTS_3857_Light.json",
  center: [-4.2, 52.4],
  zoom: 6,
  hash: true,
  maxZoom: 18,
  minZoom: 6,
  transformRequest: transformRequest(os_key),
  maxBounds: [
    [-10.7, 49.5],
    [1.9, 61.3],
  ],
  interactive: logged_in,
});
map.dragRotate.disable();
map.touchZoomRotate.disableRotation();
map.addControl(
  new maplibregl.NavigationControl({
    showCompass: false,
  }),
);

map.on("load", () => {
  if (logged_in) {
    fetchData(map);
    mapInteractions(map);
    setupFilters(map);
  }
});
