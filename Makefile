
ansible:
	ansible-playbook ansible/playbook.yml

provision:
	ansible-playbook ansible/playbook.yml --tags provision

build:
	cargo build --release

deploy: build
	ansible-playbook ansible/playbook.yml --tags deploy
