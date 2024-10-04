import {
    Grid,
    html
} from "https://unpkg.com/gridjs?module";

console.log('hello world');
let canvas = document.getElementById("myCanvas");
let ctx = canvas.getContext("2d");

const grid = new Grid({
    columns: ["Id", "Fitness"],
    data: [],
    sort: true,
    style: {
        table: {
            border: '3px solid #ccc'
        },
        th: {
            'background-color': 'black',
            color: 'white',
            'border-bottom': '3px solid #ccc',
            'text-align': 'center'
        },
        td: {
            'background-color': 'black',
            color: 'white',
            'text-align': 'center',
            'border': '1px solid #ccc',
            'text-align': 'center'
        }
    }
}).render(document.getElementById("wrapper"));

const scaling = 0.2;
const maxY = 3000;
const maxX = 7000;
const BASE_URL = 'http://localhost:3000';

ctx.canvas.width = maxX * scaling;
ctx.canvas.height = maxY * scaling;

let currentPopulation = null;
const currentTerrain = await (await fetchData("terrain")).json();
redraw();
let routeFilter = () => { return true; };

let nextButton = document.getElementById("next_button");
nextButton.onclick = async () => {
    await fetchData('next', {
        method: 'PUT',
    }).then((data) => console.log('Next generation: ', data.response)).catch((error) => console.log(error));
    await fetchDataAndHandleResponse('population', (data) => {
        currentPopulation = data;
    });
    redraw();
};


let reset_button = document.getElementById("reset_button");
reset_button.onclick = async () => {
    await fetchData('reset', {
        method: 'PUT',
    });
    redraw();
};

function printStats(population) {
    const routes_sizes = Array.from(population['routes'], (r) => r['positions'].length);
    const mean = routes_sizes.reduce((accumulator, currentValue) => accumulator + currentValue, 0) / routes_sizes.length;
    const stats = {
        "population size": population['routes'].length,
        "route sizes": {
            "max": Math.max(...routes_sizes),
            "min": Math.min(...routes_sizes),
            "mean": mean,
        }
    };
    document.getElementById("stats").value = JSON.stringify(stats);

}

async function fetchData(endpoint, options = {}) {
    const response = await fetch(`${BASE_URL}/${endpoint}`, options);
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response;
}

function redraw() {
    clearCanvas();
    if (currentPopulation) {
        drawPopulation(currentPopulation);
    }
    if (currentTerrain) {
        drawLine(currentTerrain);
    }
}

async function fetchDataAndHandleResponse(dataType, handleResponse, options = {}) {
    await (await fetchData(dataType, options)).json()
        .then((data) => {
            handleResponse(data);
            console.log(`Got ${dataType}`);
        })
        .catch((error) => console.log(error));
}

function drawPopulation(population) {
    for (const route of population.routes.filter(routeFilter)) {
        drawLine(route['positions'], 'green');
    }
    ctx.fillStyle = "white";
    ctx.font = "30px serif";
    ctx.fillText("Population id: " + population['id'], 10, 32);
    printStats(population);
    drawTable(population);
}

function clearCanvas() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
}

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

function drawTable(population) {
    grid.updateConfig({
        data: population.fitness.map((v, i) => [i, v]),
        sort: true,
    }).forceRender();
}
