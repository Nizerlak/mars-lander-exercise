window.BENCHMARK_DATA = {
  "lastUpdate": 1737206516454,
  "repoUrl": "https://github.com/Nizerlak/mars-lander-exercise",
  "entries": {
    "App signle iteration benchmark": [
      {
        "commit": {
          "author": {
            "email": "prdabr@gmail.com",
            "name": "nizerlak",
            "username": "Nizerlak"
          },
          "committer": {
            "email": "prdabr@gmail.com",
            "name": "nizerlak",
            "username": "Nizerlak"
          },
          "distinct": true,
          "id": "447f5a17061e9117c88d921132f008ee6250bd7a",
          "message": "Fix benchmarks workflow",
          "timestamp": "2025-01-18T13:55:10+01:00",
          "tree_id": "ed22067312182fed3fa05978b3edb8d1b8e4036e",
          "url": "https://github.com/Nizerlak/mars-lander-exercise/commit/447f5a17061e9117c88d921132f008ee6250bd7a"
        },
        "date": 1737205073418,
        "tool": "cargo",
        "benches": [
          {
            "name": "run_simple_sim_light_settings",
            "value": 310791,
            "range": "± 2605",
            "unit": "ns/iter"
          },
          {
            "name": "run_complicated_sim_light_settings",
            "value": 392383,
            "range": "± 6080",
            "unit": "ns/iter"
          },
          {
            "name": "run_simple_sim_hard_settings",
            "value": 2138026,
            "range": "± 62000",
            "unit": "ns/iter"
          },
          {
            "name": "run_complicated_sim_hard_settings",
            "value": 2680690,
            "range": "± 30803",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_simple_sim_light_settings",
            "value": 706657,
            "range": "± 35636",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_complicated_sim_light_settings",
            "value": 1054281,
            "range": "± 48738",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_simple_sim_hard_settings",
            "value": 5182730,
            "range": "± 184977",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_complicated_sim_hard_settings",
            "value": 8557301,
            "range": "± 299432",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "prdabr@gmail.com",
            "name": "nizerlak",
            "username": "Nizerlak"
          },
          "committer": {
            "email": "prdabr@gmail.com",
            "name": "nizerlak",
            "username": "Nizerlak"
          },
          "distinct": true,
          "id": "2ccc6249ecfb49ff4c3d0b2c3403d143a9db1d4b",
          "message": "Revert \"Compare with previous results\"\n\nThis reverts commit b82bfda0f51bc616326cd66c9c98e1e80f1de39b.",
          "timestamp": "2025-01-18T14:19:16+01:00",
          "tree_id": "ed22067312182fed3fa05978b3edb8d1b8e4036e",
          "url": "https://github.com/Nizerlak/mars-lander-exercise/commit/2ccc6249ecfb49ff4c3d0b2c3403d143a9db1d4b"
        },
        "date": 1737206516184,
        "tool": "cargo",
        "benches": [
          {
            "name": "run_simple_sim_light_settings",
            "value": 283025,
            "range": "± 4180",
            "unit": "ns/iter"
          },
          {
            "name": "run_complicated_sim_light_settings",
            "value": 398929,
            "range": "± 1456",
            "unit": "ns/iter"
          },
          {
            "name": "run_simple_sim_hard_settings",
            "value": 2126371,
            "range": "± 37938",
            "unit": "ns/iter"
          },
          {
            "name": "run_complicated_sim_hard_settings",
            "value": 2703891,
            "range": "± 9617",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_simple_sim_light_settings",
            "value": 617376,
            "range": "± 35405",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_complicated_sim_light_settings",
            "value": 1005229,
            "range": "± 53491",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_simple_sim_hard_settings",
            "value": 4908984,
            "range": "± 103424",
            "unit": "ns/iter"
          },
          {
            "name": "run_next_population_complicated_sim_hard_settings",
            "value": 8572509,
            "range": "± 191128",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}