use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
use start_match::match_runner_server::{MatchRunner, MatchRunnerServer};
use start_match::{RunnerConfig, MatchesResponse, Sar as SarView, MatchReplay as MatchReplayView, Action as ActionView};
use engine::engine::{Engine, EngineConfig, SAR};
use game_runner::worker::{ModelStore, RolloutWorker};
use game_runner::manager::RolloutConfig;
use tokio::macros::support::thread_rng_n;
use rand::{Rng, thread_rng};
use std::convert::TryInto;
use engine::state::{GameState, Action};
use engine::state::PlayerIdx::Player1;

pub mod start_match {
	tonic::include_proto!("surge_proto");
}

#[derive(Debug, Default)]
pub struct MyMatchRunner {}

#[tonic::async_trait]
impl MatchRunner for MyMatchRunner {
	async fn start_match(&self, request: Request<RunnerConfig>)
						 -> Result<Response<MatchesResponse>, Status> { // Return an instance of type HelloReply
		let config: RunnerConfig = request.into_inner();
		for agent in &config.agent_ids {
			println!("{}", agent);
		}
		println!("Got a config: {:?}", config);

		let engine = Engine::new(EngineConfig::default());

		let model_store = ModelStore::new(&"A:\\surge\\model_store\\fc_model",
										  vec![1]);
		let mut worker = RolloutWorker::new(RolloutConfig {
			engine_config: EngineConfig {
				random_seed: thread_rng().gen::<u64>(),
				fruit_density: 0.2f32,
				populate_board: true,
				board_size: 10,
			},
			agent_ids: (config.target_id.try_into().unwrap(), config.agent_ids.try_into().unwrap()),
			max_rounds: config.max_rounds.try_into().unwrap(),
			evaluation_mode: config.evaluation_mode,
			max_matches: config.max_matches.try_into().unwrap(),
		}, model_store);
		println!("Starting run until we have {} matches", config.max_matches.clone());
		let res = worker.play_matches();
		// println!("Results: {:?}", res);

		let mut view_match_res = vec![];
		for replay in res {
			let mut sars_view = vec![];
			for sar in replay.sars {
				let (gamestate, action_mask) = GameState::get_state_vec_view(&sar.gamestate, Player1);

				let sar_view = SarView {
					state: gamestate,
					reward: sar.reward,
					action: Action::as_num(sar.actions),
					action_mask: sar.action_mask.iter().map(|a| Action::as_num(*a)).collect(),
					terminal: sar.terminal
				};
				sars_view.push(sar_view);
			}
			let match_replay_view = MatchReplayView {
				sars: sars_view,
				player1_id: replay.agent_ids.0,
				player2_id: replay.agent_ids.1,
				result: replay.p1_won
			};
			view_match_res.push(match_replay_view);
		}

		// let sars = vec![Sar {
		// 	state: vec![0.32f32],
		// 	reward: 0.1f32,
		// 	action: 0,
		// 	action_mask: vec![0],
		// 	terminal: false,
		// }];
		//
		// let match_replay = MatchReplay {
		// 	sars: sars,
		// 	player1_id: 1,
		// 	player2_id: 69420,
		// 	result: true,
		// };

		let reply = MatchesResponse {
			replays: view_match_res
		};

		Ok(Response::new(reply)) // Send back our formatted greeting
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "127.0.0.1:50051".parse()?;
	let runner = MyMatchRunner::default();

	Server::builder()
		.add_service(MatchRunnerServer::new(runner))
		.serve(addr)
		.await?;

	Ok(())
}