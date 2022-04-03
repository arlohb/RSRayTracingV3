import * as Comlink from "comlink";

(async () => {
  console.log("Loading main thread wasm");
  const egui = await import("../dist/pkg/rs_ray_tracing_v2.js");

  const memory = (await egui.default()).memory;

  console.log("Creating worker");
  const worker = new Worker(new URL("./wasm-worker.js", import.meta.url), {
    type: "module"
  });

  console.log("Wrapping worker in comlink");
  const link = Comlink.wrap(worker);

  console.log("Running worker");
  await link.init(memory);
  await link.initThreadPool();

  const width = 400;
  const height = 300;
  window.rayTracerOptions = JSON.stringify({
    camera: { x: 5., y: 5., z: 5. },
    rotation: { x: 0.7, y: -Math.PI / 4., z: 0. },
    fov: 70.,
    width,
    height,
    scene: {
      "objects": [
        {
          "name": "sphere",
          "material": {
            "colour": [
              1.0,
              0.5212054252624512,
              0.0
            ],
            "specular": 5.0,
            "metallic": 1.0
          },
          "geometry": {
            "Sphere": {
              "center": {
                "x": 1.5,
                "y": 0.0,
                "z": 0.0
              },
              "radius": 1.0
            }
          }
        },
        {
          "name": "sphere",
          "material": {
            "colour": [
              1.0,
              0.3486607074737549,
              0.0
            ],
            "specular": 800.0,
            "metallic": 0.2
          },
          "geometry": {
            "Sphere": {
              "center": {
                "x": 3.1,
                "y": 0.0,
                "z": 2.1
              },
              "radius": 1.0
            }
          }
        },
        {
          "name": "sphere",
          "material": {
            "colour": [
              0.0,
              0.6445307731628418,
              1.0
            ],
            "specular": 80.0,
            "metallic": 0.0
          },
          "geometry": {
            "Sphere": {
              "center": {
                "x": -8.3,
                "y": 0.0,
                "z": 0.0
              },
              "radius": 1.0
            }
          }
        },
        {
          "name": "plane",
          "material": {
            "colour": [
              0.8000000715255737,
              0.800000011920929,
              1.0
            ],
            "specular": 50.0,
            "metallic": 0.2
          },
          "geometry": {
            "Plane": {
              "center": {
                "x": 0.0,
                "y": -1.5,
                "z": 0.0
              },
              "normal": {
                "x": 0.0,
                "y": 1.0,
                "z": 0.0
              },
              "size": 5.0
            }
          }
        }
      ],
      "lights": [
        {
          "Direction": {
            "intensity": [
              0.4,
              0.4,
              0.4
            ],
            "direction": {
              "x": -0.5345224838248488,
              "y": -0.8017837257372732,
              "z": -0.2672612419124244
            }
          }
        },
        {
          "Point": {
            "intensity": [
              0.4,
              0.4,
              0.4
            ],
            "position": {
              "x": 0.0,
              "y": 2.0,
              "z": 0.0
            }
          }
        }
      ],
      "background_colour": [
        0.5,
        0.8,
        1.0
      ],
      "ambient_light": [
        0.2,
        0.2,
        0.2
      ],
      "reflection_limit": 4,
      "do_objects_spin": false
    },
  });

  // this will only await the first one
  await link.renderImage();

  console.log("Starting egui");
  egui.start("the_canvas_id");
  document.getElementById("center_text").remove();
})();
