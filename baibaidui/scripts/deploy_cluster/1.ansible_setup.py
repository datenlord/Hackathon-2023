import os
import yaml
import argparse
import sys
import pexpect

PASSWORD="aaaaa"

def run_cmd(cmd):
    print("> "+cmd)
    os.system(cmd)


def read_yaml(f):
    # parse
    import ruamel.yaml
    yaml = ruamel.yaml.YAML(typ='rt')
    parsed_data = yaml.load(f)

    return parsed_data

def entry():
    # read cluster-nodes.yml
    with open('scripts/deploy_cluster/node_config.yaml', 'r') as f:
        run_cmd("scripts/install/install_ansible.sh")

        # write to gen_ansible.ini
        ansible="[web]\n"

        # gen ssh key if not exist
        # if not os.path.exists("/root/.ssh/id_rsa"):
        #     run_cmd("ssh-keygen -t rsa -b 2048")

        cluster_nodes = read_yaml(f)
        appeared_node={}
        for nid in cluster_nodes["nodes"]:
            node=cluster_nodes["nodes"][nid]
            ip=node["addr"].split(":")[0]
            port=node["addr"].split(":")[1]

            if ip not in appeared_node:
                ansible+="webserver{} ansible_host={} ansible_user=root\n".format(ip,ip)
                appeared_node[ip]=1

            # run_cmd("ssh root@{} 'apt install python'".format(ip))
            # run_cmd("ssh-copy-id root@{}".format(ip))
        
        # write to gen_ansible.ini
        with open("scripts/deploy_cluster/gen_ansible.ini","w") as f:
            f.write(ansible)
        

        # with open("gen_ansible.cfg","w") as f:
        #     f.write(
        #         "[defaults]\n"+\
        #         "inventory = ./gen_ansible.ini\n"+\
        #         "remote_user = root\n"+\
        #         "private_key_file = /root/.ssh/id_rsa\n"+\
        #         "host_key_checking = False"
        #     )
        
        # run ansible
        run_cmd("ansible -i scripts/deploy_cluster/gen_ansible.ini -m ping all")
        
entry()