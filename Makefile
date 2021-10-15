build:
	# Build the crates in the `examples` folder
	docker-compose build
	# Copy the files to the mounted target dir
	docker-compose up
	# Re-assign "target" dir to current user
	sudo chown -R $$USER:$$USER target

run:
	retroarch --verbose --load-menu-on-error --subsystem=blastem
