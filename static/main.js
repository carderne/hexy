const style =
  "https://raw.githubusercontent.com/OrdnanceSurvey/OS-Vector-Tile-API-Stylesheets/main/OS_VTS_3857_Light.json";

const fmtDist = (distance) =>
  distance >= 2000
    ? `${(distance / 1000).toFixed(1)} km`
    : `${distance.toFixed(0)} m`;

const fmtTime = (secs) => {
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  const remSecs = secs % 60;
  return hours === 0
    ? `${String(mins)}m ${String(remSecs)}s`
    : `${String(hours)}h ${String(mins)}m`;
};

const fmtDate = (raw) => {
  const dt = new Date(raw);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);

  if (dt.getTime() >= today.getTime()) {
    return `Today at ${dt.toLocaleString("en-UK", { hour: "numeric", minute: "numeric", hour12: true })}`;
  } else if (dt.getTime() >= yesterday.getTime()) {
    return `Yesterday at ${dt.toLocaleString("en-UK", { hour: "numeric", minute: "numeric", hour12: true })}`;
  } else if (dt.getFullYear() === now.getFullYear()) {
    return dt.toLocaleString("en-UK", { month: "long", day: "numeric" });
  } else {
    return dt.toLocaleString("en-UK", {
      month: "long",
      day: "numeric",
      year: "numeric",
    });
  }
};

const fmtActivity = (type) => {
  return (
    {
      Ride: "🚲",
      EBikeRide: "🚲",
      EMountainBikeRide: "🚲",
      GravelRide: "🚲",
      MountainBikeRide: "🚲",
      Run: "🏃",
      TrailRun: "🏃",
      Walk: "🥾",
      Hike: "🥾",
      Swim: "🏊",
    }[type] || type
  );
};

const transformRequest = (url, resourceType) => {
  if (resourceType !== "Style" && url.startsWith("https://api.os.uk")) {
    url = new URL(url);
    if (!url.searchParams.has("key")) url.searchParams.append("key", osKey);
    if (!url.searchParams.has("srs")) url.searchParams.append("srs", 3857);
    return { url: new Request(url).url };
  }
};

const makeHexes = (cells) => ({
  type: "FeatureCollection",
  features: [
    {
      type: "Feature",
      id: "hexes",
      properties: {},
      geometry: {
        type: "MultiPolygon",
        coordinates: h3.cellsToMultiPolygon(cells, true),
      },
    },
  ],
});

const activeFilters = {
  btnRide: false,
  btnRun: false,
  btnWalk: false,
  btnSwim: false,
};

const filterMap = {
  btnRide: [
    "Ride",
    "EBikeRide",
    "EMountainBikeRide",
    "GravelRide",
    "MountainBikeRide",
  ],
  btnRun: ["Run", "TrailRun"],
  btnWalk: ["Walk", "Hike"],
  btnSwim: ["Swim"],
};

const updateFilters = () => {
  let filters = ["all"];
  Object.keys(activeFilters).forEach((key) => {
    if (activeFilters[key]) {
      filterMap[key].forEach((t) => {
        filters.push(["!=", ["get", "sport_type"], t]);
      });
    }
  });
  map.setFilter("activities", filters.length > 1 ? filters : null);
};

const toggleButtonState = (id) => {
  const button = document.getElementById(id);
  activeFilters[id] = !activeFilters[id];
  button.style.opacity = activeFilters[id] ? "0.5" : "1";
  updateFilters();
};

Object.keys(activeFilters).forEach((key) => {
  const button = document.getElementById(key);
  if (button) {
    button.addEventListener("click", () => toggleButtonState(key));
  }
});

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
    [-10.7, 49.5],
    [1.9, 61.3],
  ],
});
map.dragRotate.disable();
map.touchZoomRotate.disableRotation();
map.addControl(
  new maplibregl.NavigationControl({
    showCompass: false,
  }),
);

function processData({ activities, cells }) {
  map.addSource("hex", { type: "geojson", data: makeHexes(cells) });
  map.addLayer({
    id: "hex",
    type: "fill",
    source: "hex",
    paint: {
      "fill-color": "hsla(0, 50%, 50%, 0.3)",
      "fill-outline-color": "rgba(0,0,0,0)",
    },
  });

  map.addSource("activities", { type: "geojson", data: activities });
  map.addLayer({
    id: "activities",
    type: "line",
    source: "activities",
    layout: { "line-join": "round", "line-cap": "round" },
    paint: {
      "line-width": 8,
      "line-color": [
        "case",
        [
          "any",
          ["==", ["get", "sport_type"], "EBikeRide"],
          ["==", ["get", "sport_type"], "EMountainBikeRide"],
          ["==", ["get", "sport_type"], "GravelRide"],
          ["==", ["get", "sport_type"], "MountainBikeRide"],
          ["==", ["get", "sport_type"], "Ride"],
        ],
        "#984ea3", // lilac
        [
          "any",
          ["==", ["get", "sport_type"], "Run"],
          ["==", ["get", "sport_type"], "TrailRun"],
        ],
        "#ff7f00", // orange
        [
          "any",
          ["==", ["get", "sport_type"], "Walk"],
          ["==", ["get", "sport_type"], "Hike"],
        ],
        "#4daf4a", // green
        ["==", ["get", "sport_type"], "Swim"],
        "#377eb8", // blue
        "#595959", // dark grey
      ],
      "line-opacity": ["interpolate", ["linear"], ["zoom"], 7, 0.6, 15, 0.5],
      "line-width": ["interpolate", ["linear"], ["zoom"], 7, 2, 15, 6],
    },
  });
}

const fetchData = () => {
  document.getElementById("loading").style.display = "flex";
  fetch("/data")
    .then((res) => {
      if (!res.ok) throw new Error("Failed to load /data");
      return res.json();
    })
    .then(processData)
    .catch((err) => {
      console.error("Failed to load /data", err);
    })
    .finally(() => {
      document.getElementById("loading").style.display = "none";
    });
};

map.on("load", () => {
  fetchData();
  map.on("click", "activities", (e) => {
    e.preventDefault();
    const props = e.features?.[0]?.properties;
    document.getElementById("p-id").href =
      `https://www.strava.com/activities/${props.id}`;
    document.getElementById("p-name").innerText = props.name;
    document.getElementById("p-date").innerText = fmtDate(props.start_date);
    document.getElementById("p-distance").innerText = fmtDist(props.distance);
    document.getElementById("p-moving").innerText = fmtTime(props.moving_time);
    document.getElementById("p-type").innerText = fmtActivity(props.sport_type);

    document.getElementById("props").style.display = "block";
  });

  map.on("click", function (e) {
    if (e.defaultPrevented === false) {
      document.getElementById("props").style.display = "none";
    }
  });

  map.on("mouseenter", "activities", () => {
    map.getCanvas().style.cursor = "pointer";
  });

  map.on("mouseleave", "activities", () => {
    map.getCanvas().style.cursor = "";
  });
});
