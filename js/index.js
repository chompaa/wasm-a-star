const WIDTH = 15;
const HEIGHT = 15;
const CELL_SIZE = 32;
const NODE_SIZE = 24;
const WALL_COLOUR = "#000000";
const DEFAULT_COLOUR = "#F7FFF7";
const START_COLOUR = "#FFE66D";
const END_COLOUR = "#FF6B6B";

const canvas = document.getElementById("a-star-canvas");
canvas.width = (CELL_SIZE + 1) * WIDTH + 1;
canvas.height = (CELL_SIZE + 1) * HEIGHT + 1;
const ctx = canvas.getContext('2d');

let startKeyHeld = false
let endKeyHeld = false
let mouseX = 0;
let mouseY = 0;

let wasm;

import("../pkg/index_bg").then((mod) => {
  wasm = mod;
})

document.addEventListener("keydown", () => {
  if (event.keyCode == 83) {
    startKeyHeld = true;
  } else if (event.keyCode == 69) {
    endKeyHeld = true;
  }
})

document.addEventListener("keyup", () => {
  if (event.keyCode == 83) {
    startKeyHeld = false;
  } else if (event.keyCode == 69) {
    endKeyHeld = false;
  }
})

import("../pkg/index.js").then((lib) => {
  const grid = lib.Grid.new(WIDTH, HEIGHT);
  grid.set_start(1, 1);
  grid.set_end(WIDTH - 2, HEIGHT - 2);
  const states = lib.NodeStates;

  const drawNodes = () => {
    const nodes = new Uint8Array(wasm.memory.buffer, grid.nodes(), WIDTH * HEIGHT);

    ctx.beginPath();

    for (let y = 0; y < HEIGHT; y++) {
      for (let x = 0; x < WIDTH; x++) {
        switch (nodes[y * WIDTH + x]) {
          case states.DEFAULT:
            ctx.fillStyle = DEFAULT_COLOUR;
            break;
          case states.WALL:
            ctx.fillStyle = WALL_COLOUR;
            break;
          case states.START:
            ctx.fillStyle = START_COLOUR;
            break;
          case states.END:
            ctx.fillStyle = END_COLOUR;
            break;
          case states.PATH:
            ctx.fillStyle = PATH_COLOUR;
            break;
        }

        ctx.lineWidth = 2;
        ctx.strokeRect(
          ((x * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1) - (NODE_SIZE / 2),
          ((y * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1) - (NODE_SIZE / 2),
          NODE_SIZE,
          NODE_SIZE
        );

        ctx.lineWidth = 0;
        ctx.fillRect(
          ((x * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1) - (NODE_SIZE / 2),
          ((y * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1) - (NODE_SIZE / 2),
          NODE_SIZE,
          NODE_SIZE
        );
      }
    }

    ctx.stroke();
  };

  const reloadCanvas = () => {
    grid.clear_path();
    ctx.fillRect(0, 0, WIDTH * (CELL_SIZE + 1), HEIGHT * (CELL_SIZE + 1));
    drawNodes();
  }


  canvas.addEventListener("mousemove", (event) => {
    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const x = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), HEIGHT - 1);
    const y = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), WIDTH - 1);

    if (x === mouseX && y === mouseY || x < 0 || y < 0) {
      return;
    }

    mouseX = x
    mouseY = y

    if (event.which && !startKeyHeld && !endKeyHeld) {
      if (event.ctrlKey) {
        grid.remove_wall(mouseX, mouseY);
      } else {
        grid.add_wall(mouseX, mouseY);
      }

      reloadCanvas();
    }
  });

  canvas.addEventListener("mousedown", (event) => {
    if (startKeyHeld) {
      grid.set_start(mouseX, mouseY);
    } else if (endKeyHeld) {
      grid.set_end(mouseX, mouseY);
    } else if (event.ctrlKey) {
      grid.remove_wall(mouseX, mouseY);
    } else {
      grid.add_wall(mouseX, mouseY);
    }

    reloadCanvas();
    drawNodes();
  });

  document.getElementById("start").addEventListener("click", () => {
    console.log("sup!")
    grid.clear_path();
    grid.a_star()

    ctx.beginPath();
    ctx.strokeStyle = WALL_COLOUR;
    ctx.lineWidth = 3;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    const path = new Uint16Array(wasm.memory.buffer, grid.get_path(), grid.get_path_count())
    path.forEach((node, index) => {
      if (path[index + 1]) {
        const curX = path[index] % WIDTH;
        const curY = Math.floor(path[index] / WIDTH);
        const nextX = path[index + 1] % WIDTH;
        const nextY = Math.floor(path[index + 1] / WIDTH);

        ctx.moveTo((curX * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1, (curY * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1);
        ctx.lineTo((nextX * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1, (nextY * (CELL_SIZE + 1)) + (CELL_SIZE / 2) + 1);
      }
    });

    ctx.stroke();
  });

  document.getElementById("keybinds").addEventListener("click", () => {
    alert("Start: S + Click\nEnd: E + Click\nWall (add): Click\nWall (remove): Right-click")
  });

  drawNodes();
})

