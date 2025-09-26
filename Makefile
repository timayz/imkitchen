dev:
	cargo run -- --log error,imkitchen=debug,evento=debug migrate -c ./imkitchen.toml
	cargo watch -x 'run -- --log error,imkitchen=debug,evento=debug serve -c ./imkitchen.toml'

tailwind:
	tailwindcss -i ./tailwind.css -o ./crates/imkitchen-web/static/css/main.css --watch

reset:
	cargo run -- --log error,imkitchen=debug,evento=debug reset -c ./imkitchen.toml

cert:
	mkdir -p .docker/traefik/certs
	mkcert -install
	mkcert -key-file .docker/traefik/certs/imkitchen.key -cert-file .docker/traefik/certs/imkitchen.crt imkitchen.localhost traefik.localhost *.imkitchen.localhost

up:
	sudo docker compose up -d --remove-orphans

stop:
	sudo docker compose stop

down:
	sudo docker compose down -v --rmi local --remove-orphans

lint:
	cargo clippy --fix --workspace --all-features -- -D warnings

test:
	cargo test

e2e:
	npx playwright test --headed

fmt:
	cargo fmt -- --emit files

machete:
	cargo machete

advisory.clean:
	rm -rf ~/.cargo/advisory-db

pants: advisory.clean
	cargo pants

audit: advisory.clean
	cargo audit

outdated:
	cargo outdated
