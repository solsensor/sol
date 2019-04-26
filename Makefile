
ansible:
	ansible-playbook ansible/playbook.yml

provision:
	ansible-playbook ansible/playbook.yml --tags provision

deploy:
	ansible-playbook ansible/playbook.yml --tags deploy
