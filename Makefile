
ansible:
	ansible-playbook ansible/playbook.yml

ansible-deps:
	ansible-galaxy install -r ansible/requirements.yml --force

provision:
	ansible-playbook ansible/playbook.yml --tags provision

deploy:
	ansible-playbook ansible/playbook.yml --tags deploy
