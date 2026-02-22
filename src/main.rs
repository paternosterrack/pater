#![doc = include_str!("../README.md")]

use clap::Parser;

mod cli;
mod commands;
mod domain;
mod rack;
mod services;

use cli::{Cli, Commands};
use commands::{
    handle_author_commands, handle_rack_commands, handle_runtime_commands, handle_trust_commands,
};
use domain::models::{MarketRef, State};
use services::marketplace::{
    checked_load_marketplace, dedupe_markets, ensure_default_marketplace, load_policy,
};
use services::storage::load_state;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    if let Err(error) = run(cli) {
        if json {
            print_json_error(&error.to_string());
        } else {
            eprintln!("error: {}", error);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let mut state = load_state()?;
    let policy = load_policy()?;

    ensure_default_marketplace(&mut state)?;

    if handle_trust_commands(&cli, &policy)? {
        return Ok(());
    }
    if handle_rack_commands(&cli)? {
        return Ok(());
    }
    if handle_author_commands(&cli)? {
        return Ok(());
    }

    refresh_configured_marketplaces(&state, &cli.marketplace);

    let (all_markets, default_market) = if command_requires_default_marketplace(&cli.command) {
        let default_market = checked_load_marketplace(&cli.marketplace, &policy)?;
        let all_markets = build_market_context(&state, &default_market.name, &cli.marketplace);
        (all_markets, Some(default_market))
    } else {
        let mut all_markets = state.marketplaces.clone();
        all_markets.extend(state.installed.iter().map(|p| MarketRef {
            name: p.marketplace.clone(),
            source: p.marketplace_source.clone(),
        }));
        dedupe_markets(&mut all_markets);
        (all_markets, None)
    };

    handle_runtime_commands(
        &cli,
        &mut state,
        &policy,
        &all_markets,
        default_market.as_ref(),
    )
}

fn refresh_configured_marketplaces(state: &State, default_source: &str) {
    for market in &state.marketplaces {
        let _ = rack::refresh_marketplace(&market.source);
    }
    let _ = rack::refresh_marketplace(default_source);
}

fn build_market_context(state: &State, default_name: &str, default_source: &str) -> Vec<MarketRef> {
    let mut all_markets = vec![MarketRef {
        name: default_name.to_string(),
        source: default_source.to_string(),
    }];
    all_markets.extend(state.marketplaces.clone());
    dedupe_markets(&mut all_markets);
    all_markets
}

fn command_requires_default_marketplace(command: &Commands) -> bool {
    matches!(
        command,
        Commands::Search { .. }
            | Commands::Recommend { .. }
            | Commands::Plan { .. }
            | Commands::Show { .. }
            | Commands::Install { .. }
            | Commands::Apply { .. }
            | Commands::Hook { .. }
            | Commands::Validate
            | Commands::Ensure { .. }
            | Commands::Policy { .. }
    )
}

fn print_json_error(message: &str) {
    let out = serde_json::json!({
        "ok": false,
        "error": {
            "code": map_error_code(message),
            "message": message,
            "hint": error_hint(message),
            "retryable": false
        },
        "meta": {"version": "v1"}
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{\"ok\":false}".to_string())
    );
}

fn map_error_code(msg: &str) -> &'static str {
    let m = msg.to_ascii_lowercase();
    if m.contains("policy") {
        "POLICY_DENY"
    } else if m.contains("signature") {
        "SIGNATURE_INVALID"
    } else if m.contains("not found") {
        "NOT_FOUND"
    } else if m.contains("permission") {
        "PERMISSION_DELTA_BLOCKED"
    } else {
        "INTERNAL_ERROR"
    }
}

fn error_hint(msg: &str) -> &'static str {
    let m = msg.to_ascii_lowercase();
    if m.contains("signature") {
        "run `pater trust init` and verify marketplace.sig"
    } else if m.contains("policy") {
        "review ~/.config/pater/policy.toml"
    } else if m.contains("not found") {
        "check plugin/marketplace name and run `pater search`"
    } else {
        "run `pater --json check` for diagnostics"
    }
}
