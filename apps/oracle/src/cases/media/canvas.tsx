const WIDTH = 592;
const HEIGHT = 312;
const NOISE_SEED = 12345;

function nextRandom(state: { value: number }): number {
  state.value = (Math.imul(state.value, 48271) >>> 0) % 2147483647;
  return state.value / 2147483647;
}

function drawShapes(context: CanvasRenderingContext2D) {
  context.fillStyle = "#f97316";
  context.fillRect(10, 10, 120, 80);
  context.strokeStyle = "#f8fafc";
  context.lineWidth = 4;
  context.strokeRect(150, 10, 120, 80);
  context.beginPath();
  context.arc(350, 50, 40, 0, Math.PI * 1.5);
  context.closePath();
  context.fillStyle = "#0ea5e9";
  context.fill();
  context.beginPath();
  context.moveTo(410, 90);
  context.bezierCurveTo(440, 0, 500, 0, 520, 90);
  context.quadraticCurveTo(550, 40, 580, 90);
  context.lineWidth = 3;
  context.strokeStyle = "#22c55e";
  context.lineCap = "round";
  context.stroke();
}

function drawGradients(context: CanvasRenderingContext2D) {
  const linear = context.createLinearGradient(10, 0, 190, 0);
  linear.addColorStop(0, "#7c3aed");
  linear.addColorStop(1, "#f43f5e");
  context.fillStyle = linear;
  context.fillRect(10, 110, 180, 80);
  const radial = context.createRadialGradient(290, 150, 5, 290, 150, 60);
  radial.addColorStop(0, "#facc15");
  radial.addColorStop(1, "rgba(250,204,21,0)");
  context.fillStyle = radial;
  context.fillRect(210, 110, 160, 80);
  context.save();
  context.globalAlpha = 0.6;
  context.globalCompositeOperation = "screen";
  context.translate(470, 150);
  context.rotate(0.3);
  context.fillStyle = "#0ea5e9";
  context.fillRect(-60, -30, 120, 60);
  context.restore();
}

function drawText(context: CanvasRenderingContext2D) {
  context.font = "700 32px Inter";
  context.fillStyle = "#f8fafc";
  context.textBaseline = "alphabetic";
  context.fillText("Canvas text", 10, 240);
  context.font = "400 18px JetBrains Mono";
  context.fillStyle = "#94a3b8";
  context.fillText(`width ${Math.round(context.measureText("Canvas text").width)}`, 10, 270);
  context.shadowColor = "rgba(0,0,0,0.8)";
  context.shadowBlur = 8;
  context.shadowOffsetY = 4;
  context.fillStyle = "#f97316";
  context.font = "800 28px Inter";
  context.fillText("Shadowed", 220, 250);
  context.shadowColor = "transparent";
}

function drawNoise(context: CanvasRenderingContext2D) {
  const pixels = context.createImageData(160, 90);
  const state = { value: NOISE_SEED };
  for (let i = 0; i < pixels.data.length; i += 4) {
    const grey = Math.floor(nextRandom(state) * 255);
    pixels.data[i] = grey;
    pixels.data[i + 1] = grey;
    pixels.data[i + 2] = grey;
    pixels.data[i + 3] = 255;
  }
  context.putImageData(pixels, 420, 210);
}

function draw(canvas: HTMLCanvasElement | null) {
  const context = canvas?.getContext("2d");
  if (!context) return;
  context.clearRect(0, 0, WIDTH, HEIGHT);
  drawShapes(context);
  drawGradients(context);
  drawText(context);
  drawNoise(context);
}

export function Case() {
  return (
    <canvas
      ref={draw}
      width={WIDTH}
      height={HEIGHT}
      style={{ position: "absolute", left: 24, top: 24 }}
    />
  );
}
