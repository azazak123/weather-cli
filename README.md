# Weather CLI

Simple CLI application that allows getting weather information about the current or future date using different providers.

## General description

This application can use WeatherApi and OpenWeather as weather providers. For using each of them, you should have a corresponding API key that you should set up in CLI. Default provider and API keys are saved in the config.json file.

## Setting provider

For the first time, you should set up the provider and corresponding API key.

For example, we want to set up WeatherApi as the provider.

```bash
cargo run -- configure --provider WeatherApi --api-key <YOUR_API_KEY>
```

Now, we can get the current weather for the specific address.
If the address contains several words, wrap it with "".

Note that now available only WeatherApi and OpenWeather providers.

## Getting forecast

To get the forecast for the specific date, you can simply add the date parameter in the format (dd.mm.yyyy):

```bash
cargo run -- get --address <YOUR_ADDRESS> --date <YOUR_DATE>
```

Note that different providers have different limits on how far the date can be. For example, WeatherApi can get forecasts for up to 13 days, and OpenWeather - for up to 5 days.

## Change provider

If you had configured a few providers, you might want to change between them. To do it you can use the command for setting up the provider without API key argument.

```bash
cargo run -- configure --provider <YOUR_PROVIDER_NAME>
```

After this, the 'get' command will use the provider that you have already set.
