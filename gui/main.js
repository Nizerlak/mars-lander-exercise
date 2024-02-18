console.log('hello world');
var c = document.getElementById("myCanvas");

var nextButton = document.getElementById("next_button");
nextButton.onclick = async () => {
    fetch('http://0.0.0.0:3000/next')
};

var ctx = c.getContext("2d");

const scaling = 0.2;
const maxY = 3000;
const maxX = 7000;

ctx.canvas.width = maxX * scaling;
ctx.canvas.height = maxY * scaling;


fetch('http://0.0.0.0:3000/terrain')
    .then(async (response) => {
        console.log("Got terrain");
        drawLine(await response.json());
    });

fetch('http://0.0.0.0:3000/routes')
    .then(async (response) => {
        console.log("Got routes");
        const routes = await response.json();
        for (const route of routes) {
            drawLine(route['positions'], 'green');
        }
    });

function canvasPoint(x, y, scale = 1) {
    return [x * scale, (maxY - y) * scale]
}

function drawLine(line, style = 'white') {
    let [x, y] = line[0]
    const [sx, sy] = canvasPoint(x, y, scaling);
    ctx.beginPath();
    ctx.strokeStyle = style;
    ctx.moveTo(sx, sy);
    for (const [x, y] of line) {
        const [sx, sy] = canvasPoint(x, y, scaling);
        ctx.lineTo(sx, sy);
    }
    ctx.stroke();
    ctx.closePath();
}

