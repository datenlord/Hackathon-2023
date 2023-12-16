import yaml
import os
import subprocess

def stdout(command):
    print("> " + command)
    try:
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True)
        # 打印标准错误
        if result.stderr:
            print("\nStandard Error:")
            print(result.stderr)
            exit(1)
        return result.stdout.strip().decode('utf-8')
    except Exception as e:
        print("Error:", e)
        exit(1)


# Get the directory of the current script
DEPLOY_CLUSTER_DIR = os.path.dirname(os.path.abspath(__file__))
NODE_CONFIG = os.path.join(DEPLOY_CLUSTER_DIR, 'node_config.yaml')
PRJ_NAME=stdout("python3 "+os.path.join(DEPLOY_CLUSTER_DIR,"../saaf/project_name.py"))

print("PRJ_NAME",PRJ_NAME)

def read_yaml(file_path):
    with open(file_path, 'r') as file:
        data = yaml.safe_load(file)
    return data


def generate_docker_compose(ip, nodes):
    services = {}
    
    for key, node in nodes.items():
        service_name = f"node{key}"

        external_port1 = int(node['addr'].split(':')[-1])
        external_port2 = external_port1 + 1
        external_ip = node['addr'].split(':')[0]

        services[service_name] = {
            'image': f'{PRJ_NAME}:v1',
            'ports': [f"{external_ip}:{external_port1}:{external_port1}/udp", f"{external_port2}:{external_port2}"],
            'deploy':{
                'resources':{
                    'limits':{
                        'memory': '6G'
                    }
                }
            },
            'volumes': [f'/root/{PRJ_NAME}_deploy/files:/etc/{PRJ_NAME}/files'],
            'environment': {
                'SYS_NODEID': key
            },
            'privileged': True # for tc control
        }

    compose_data = {'version': '3', 'services': services}
    compose_file_name = os.path.join(DEPLOY_CLUSTER_DIR, f"compose_{ip}.yml")

    with open(compose_file_name, 'w') as file:
        yaml.dump(compose_data, file, default_flow_style=False)


def main():
    data = read_yaml(NODE_CONFIG)

    grouped_nodes = {}
    for key, node in data['nodes'].items():
        ip = node['addr'].split(':')[0]
        if ip not in grouped_nodes:
            grouped_nodes[ip] = {}
        grouped_nodes[ip][key] = node

    for ip, nodes in grouped_nodes.items():
        generate_docker_compose(ip, nodes)


if __name__ == "__main__":
    main()
