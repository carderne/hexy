const h3Level = 9;
const style = "https://raw.githubusercontent.com/OrdnanceSurvey/OS-Vector-Tile-API-Stylesheets/main/OS_VTS_3857_Light.json";

const transformRequest = (url, resourceType) => {
  if(resourceType !== "Style" && url.startsWith("https://api.os.uk")) {
      url = new URL(url);
      if(!url.searchParams.has("key")) url.searchParams.append("key", osKey);
      if(!url.searchParams.has("srs")) url.searchParams.append("srs", 3857);
      return { url: new Request(url).url }
  }
};

const hexes = {
  type: "FeatureCollection",
  features: cells.map((c, i) => ({
    type: "Feature",
    id: i,
    properties: { index: c },
    geometry: {
      type: "MultiPolygon",
      coordinates: h3.cellsToMultiPolygon([c], true),
    },
  })),
};

const map = new maplibregl.Map({
  container: "map",
  style,
  center: [-4.2, 52.4],
  zoom: 6,
  hash: true,
  maxZoom: 18,
  minZoom: 6,
  transformRequest,
  maxBounds: [
    [ -10.7, 49.5 ],
    [ 1.9, 61.3 ],
  ],
});

map.dragRotate.disable();
map.touchZoomRotate.disableRotation();
map.addControl(new maplibregl.NavigationControl({
    showCompass: false
}));

map.on("load", () => {
  map.addSource("hex", {
    type: "geojson",
    data: hexes,
  });
  map.addLayer({
    id: "hex",
    type: "fill",
    source: "hex",
    paint: {
      "fill-color": "hsla(0, 50%, 50%, 0.3)",
      "fill-outline-color": "rgba(0,0,0,0)",
    },
  });

  map.addSource("activities", { "type": "geojson", data });
  map.addLayer({
    "id": "activities",
    "type": "line",
    "source": "activities",
    "layout": { "line-join": "round", "line-cap": "round" },
    "paint": { 
      "line-width": 8,
      "line-color": [
        "case",
        ["==", ["get", "type"], "Ride"], "#984ea3", // lilac
        ["==", ["get", "type"], "Run"], "#ff7f00", // orange
        ["==", ["get", "type"], "Walk"], "#4daf4a", // green
        ["==", ["get", "type"], "Hike"], "#4daf4a", // green
        ["==", ["get", "type"], "Swim"], "#377eb8", // blue
        "#595959" // dark grey
      ],
      "line-opacity": [
        "interpolate",
        ["linear"],
        ["zoom"],
        7, 0.6,
        15, 0.5,
      ],
      "line-width": [
        "interpolate",
        ["linear"],
        ["zoom"],
        7, 2,
        15, 6,
      ],
    },
  });

  map.on("click", "activities", (e) => {
    e.preventDefault();
    const ft = e.features[0];

    document.getElementById("p-id").href = `https://www.strava.com/activities/${ft.properties.id}`;
    document.getElementById("p-name").innerText = ft.properties.name;
    document.getElementById("p-date").innerText = ft.properties.start_date;
    document.getElementById("p-distance").innerText = ft.properties.distance;
    document.getElementById("p-moving").innerText = ft.properties.moving_time;
    document.getElementById("p-type").innerText = ft.properties.type;
    document.getElementById("p-sport-type").innerText = ft.properties.sport_type;

    const div = document.getElementById("props");
    div.style.display = "block";
  });

  map.on("click", function(e) {
    if (e.defaultPrevented === false) {
      const div = document.getElementById("props");
      div.style.display = "none";
    }
  });

  map.on("mouseenter", "activities", () => {
    map.getCanvas().style.cursor = "pointer";
  });

  map.on("mouseleave", "activities", () => {
    map.getCanvas().style.cursor = "";
  });
});
