use futures::{stream, StreamExt};
use reqwest::{Client, Response};
use std::{io, io::prelude::Write, process::exit, string::String, time::Instant};
use tokio;

const BASE_URL: &str = "https://ggez.ch/";
const PARALLEL_REQUESTS: usize = 45; // threads + 13

#[tokio::main]
async fn main() {
	let ascii_lowercase = vec![
		"a", "b", "c", "d", "e", "f", "g", "h", "i",
		"j", "k", "l", "m", "n", "o", "p", "q", "r",
		"s", "t", "u", "v", "w", "x", "y", "z"
	];

	let permutations = get_permutations_with_repetitions(ascii_lowercase, 4);
	let permutations_len = permutations.len();

	let mut key_urls: Vec<String> = Vec::new();
	for permutation in permutations {
		key_urls.push(format!("{}{}", BASE_URL, permutation));
	}

	println!("You're about to throw {} urls at {} ...", permutations_len, BASE_URL);
	loop {
		print!("SURE? (y/n) ");
		let _ = io::stdout().flush();

		let mut answer = String::new();

		io::stdin()
			.read_line(&mut answer)
			.expect("Failed to read line!");

		match answer.trim() {
			"y" => break, // break the loop and continue exec
			"n" => exit(1), // exit the bin
			_ => continue // cont. the loop
		}
	}

	let start_requests = Instant::now();
	request_urls(key_urls).await;
	let duration_requests = start_requests.elapsed();
	println!("Ran {} urls against {} in {:?}!", permutations_len, BASE_URL, duration_requests);
}

/*
	Generates a sequence of permutations with repetitions of n elements drawn from a choice of k values.
	Get elements^number_values (n^k) permutations with repetitions where k >= 2.
*/
fn get_permutations_with_repetitions(elements: Vec<&str>, number_values: usize) -> Vec<String> {
	let perms: Vec<_> = (2..number_values).fold(
		elements
			.iter()
			.map(|c| elements.iter().map(move |&d| d.to_owned() + *c))
			.flatten()
			.collect(),
		|acc, _| {
			acc.into_iter()
				.map(|c| elements.iter().map(move |&d| d.to_owned() + &*c))
				.flatten()
				.collect()
		},
	);
	perms
}

/*
	Asynchronous parallel requests by spawning tokio tasks.
	Clone a shared request::Client for each task.
*/
async fn request_urls(urls: Vec<String>) {
	let client = Client::new();

	let responses = stream::iter(urls)
		.map(|url| {
			let client = client.clone();
			tokio::spawn(async move { client.get(url).send().await })
		})
		.buffer_unordered(PARALLEL_REQUESTS);

	responses
		.for_each(|res| async {
			match res {
				Ok(Ok(res)) => parse_response(res),
				Ok(Err(err)) => eprintln!("reqwest::Error: {}", err),
				Err(err) => eprintln!("tokio::JoinError: {}", err),
			}
		})
		.await;
}

/*
	TODO: Handle the response
*/
fn parse_response(response: Response) {
	let mut called_url = String::new();

	match response.url().domain() {
		Some(domain) => called_url.push_str(domain),
		None => println!("Invalid domain!")
	}

	called_url.push_str(response.url().path());
	println!("{} - {}", called_url, response.status());
}