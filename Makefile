
.PHONY: ansible
ansible:
	ansible-playbook ansible/playbook.yml

deploy:
	ansible-playbook ansible/playbook.yml --skip-tags provision
