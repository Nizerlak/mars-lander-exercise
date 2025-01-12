console.log('hello world');
let canvas = document.getElementById("myCanvas");
let ctx = canvas.getContext("2d");

function setRouteState(routeId) {
    let text_box = document.getElementById("route_state");
    if (routeId == null) {
        text_box.value = "";
    } else {
        text_box.value = `Flight State: ${JSON.stringify(currentPopulation.routes[routeId].flight_state)}, Positions Length: ${currentPopulation.routes[routeId].positions.length}`;
    }
}

// create handler function
function myRowClickedHandler(event) {
    const routeId = event.data.Id;
    const commands = currentPopulation.commands[routeId];
    const commandsAcc = currentPopulation.commands_accumulated[routeId];
    setRouteState(routeId);
    routeGridApi.setGridOption('rowData', commands.angles.map((v, i) => { return { angle: v, thrust: commands.thrusts[i], thrustAcc: commandsAcc.thrusts[i], angleAcc: commandsAcc.angles[i] }; }));
    routeFilter = (_, i) => i === routeId;
    redraw();
}

const gridOptions = {
    // Row Data: The data to be displayed.
    rowData: [
    ],
    onRowClicked: myRowClickedHandler,
    // Column Definitions: Defines the columns to be displayed.
    columnDefs: [
        { field: "Id" },
        { field: "Fitness" },
        { field: "Result" }
    ]
};

// Your Javascript code to create the Data Grid
const fitnessGrid = document.querySelector('#fitnessGrid');
let fitnessGridApi = agGrid.createGrid(fitnessGrid, gridOptions);

const routeGridOptions = {
    // Row Data: The data to be displayed.
    rowData: [
    ],
    // Column Definitions: Defines the columns to be displayed.
    columnDefs: [
        {
            headerName: "Accumulated",
            children: [{ headerName: "Angle", field: "angleAcc", sortable: false, columnGroupShow: "open" }, { headerName: "Thrust", field: "thrustAcc", sortable: false, columnGroupShow: "open" }],
        },
        {
            headerName: "Commands",
            children: [{ headerName: "Angle", field: "angle", sortable: false }, { headerName: "Thrust", field: "thrust", sortable: false }],
        },
    ]
};

// Your Javascript code to create the Data Grid
const routeGrid = document.querySelector('#routeGrid');
let routeGridApi = agGrid.createGrid(routeGrid, routeGridOptions);

const scaling = 0.2;
const maxY = 3000;
const maxX = 7000;
const BASE_URL = 'http://localhost:3000';

ctx.canvas.width = maxX * scaling;
ctx.canvas.height = maxY * scaling;

let currentPopulation = null;
const currentTerrain = await(await fetchData("terrain")).json();
redraw();
const nop = () => { return true; };
let routeFilter = nop;
let run_switch = null;
function try_clear_run() {
    if (run_switch) {
        clearInterval(run_switch);
    }
}

async function handleNext() {
    await(await fetchData('next', {
        method: 'PUT',
    })).text().then(async (data) => {
        if (data === "true") {
            console.log('Got solution');
            try_clear_run();
        } else if (data !== "false") {
            throw new Error('Unexpected response for next:', data);
        }
    }).catch((error) => console.log(error));
    await fetchDataAndHandleResponse('population', (data) => {
        currentPopulation = data;
        console.log("Population");
        console.log(currentPopulation);
    });
    redraw();
    drawFitnessTable(currentPopulation);
}

let nextButton = document.getElementById("next_button");
nextButton.onclick = async () => {
    try_clear_run();
    await handleNext();
};


let reset_button = document.getElementById("reset_button");
reset_button.onclick = async () => {
    try_clear_run();
    await fetchData('reset', {
        method: 'PUT',
    });
    await fetchDataAndHandleResponse('population', (data) => {
        currentPopulation = data;
    });
    redraw();
    drawFitnessTable(currentPopulation);
};

let reset_filter_button = document.getElementById("reset_filter");
reset_filter_button.onclick = async () => {
    try_clear_run();
    routeFilter = nop;
    setRouteState(null);
    redraw();
};


let run_until_solution_button = document.getElementById("until_solution_found");
const RUN_UNTIL_INTERVAL_MS = 300;
run_until_solution_button.onclick = async () => {
    try_clear_run();
    run_switch = setInterval(handleNext, RUN_UNTIL_INTERVAL_MS);
}

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
        const color = route.flight_state == "LandedCorrectly" ? "red" : "green";
        drawLine(route['positions'], color);
    }
    ctx.fillStyle = "white";
    ctx.font = "30px serif";
    ctx.fillText("Population id: " + population['id'], 10, 32);
    printStats(population);
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

function drawFitnessTable(population) {
    const fitnesses = population.fitness.map((v, i) => { return { Id: i, Fitness: v, Result: JSON.stringify(population.routes[i].flight_state) }; })
    fitnessGridApi.setGridOption('rowData', fitnesses);
}
