### Multi Nodes

1. First clone this project on master node

2. Ansible install with `scripts/install/install_ansible.sh` (need python)

3. Config the `scripts/deploy_cluster/node_config.yaml` 

4. Set up ssh interflow and ansible node info `python scripts/deploy_cluster/1.ansible_setup.py`

5. Redploy `2.redeploy.sh`
