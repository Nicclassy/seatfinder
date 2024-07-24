# seatfinder

Finds allocations that match queries from a JSON file. The JSON file must be named `config.json` as of now. No CLI/GUI exists yet.

An example `config.json` is shown below:

```json
{
    "parity": "even",
    "headless": false,
    "queries": [
        {
            "unit_code": "CMPN1611",
            "day": 4,
            "semester": 1,
            "activity_type": "Seminar",
            "activity": 2
        },
        {
            "unit_code": "NEUR2001",
            "day": "Tuesday",
            "semester": 2,
            "activity_type": "Tutorial",
            "activity": 4,
            "start_after": "11:00"
        }
    ]
}
```


## Optional `config.json` attributes

`run_chromedriver` (default `false`): Specifies whether or not the program runs a chromedriver instance. You are expected to run an instance of chromedriver if this is set to `false`.

`headless` (default `false`): Run `chromedriver` in headless mode. Does not do anything if the chromedriver instance is run separately.

`port` (default `9515`): The port the chromedriver instance is running on.

`parity`: Determines which of the two public timetables to use (`even` or `odd`). The default value depends on the current year—if the current year is an odd number, the parity is `odd` and if the current year is an even number, the parity is `even`.

## Required `config.json` attributes

`query` or `queries`: `query` expects a single query whereas `queries` expects an array of queries. If both are specified, the value of `query` takes precedence.

## Query format

`unit_code`: The code of the unit to search for.

`day`: The day the allocation runs on. Can be an ISO week date (i.e. `1-7`) or the weekday's (abbreviated) name.

`semester` (optional): The semester that the unit is offered in. Must be `0`, `1` or `2`. `0` works the same as not specifying the semester—the first matching offering is chosen.

`activity_type`: The type of activity to search for. Must be one of the activity types specified on the [public timetable website](https://timetable.sydney.edu.au/even/timetable/#subjects).

`activity`: The number of the activity to search for.

`start_after` (optional): The time the activity starts after or starts at.