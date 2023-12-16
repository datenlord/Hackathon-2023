import yaml
import random
import string
import os

# 读取node_config.yaml
with open('files/node_config.yaml', 'r') as f:
    config = yaml.safe_load(f)

# 设置随机数种子
random_seed = config['random_seed']
random.seed(random_seed)

# 创建存放文件的文件夹
files_folder = 'files/datas'
os.makedirs(files_folder, exist_ok=True)

# 根据random_seed生成10个0.8-1.2G左右的文件
file_map = {}
for i in range(10):
    file_name = ''.join(random.choices(string.ascii_lowercase, k=5)) + '.txt'
    file_size = random.uniform(0.8, 1.2) * 1024 *50  # 文件大小在0.8-1.2G之间
    block_count = int(file_size / config['block_size'])
    file_map[file_name] = block_count

    # 实际生成文件
    with open(f"{files_folder}/"+file_name, 'wb') as file:
        file.write(os.urandom(config['block_size'] * block_count * 1024))
# 将文件map写入文件
with open('files/file_map.yaml', 'w') as f:
    yaml.dump(file_map, f)

# 过滤出spec为fs的node
fs_nodes = [node_id for node_id, node in config['nodes'].items() if 'fs' in node['spec']]

# 生成user yaml
user_yaml = {
    
}
for node_id in fs_nodes:
    num_targets = random.randint(1, len(file_map))
    targets = random.sample(file_map.keys(), k=num_targets)
    access_mode = random.choice(['loop', 'random'])
    user_yaml[node_id] = {'targets': targets, 'access': access_mode}

# 将user yaml写入文件
with open('files/user.yaml', 'w') as f:
    f.write("""
8:
    access: loop
    targets:
        - mykbe.txt
        - hrtqr.txt
6:
    access: loop
    targets:
        - cyrsk.txt
        - ngwrj.txt
3:
    access: loop_parallel
    targets:
        - zytsk.txt
        - hbuzi.txt
        - mykbe.txt
        - hrtqr.txt
        - cyrsk.txt
        - ngwrj.txt
4:
    access: loop_parallel
    targets:
        - rndva.txt
        - ngwrj.txt
        - jhdeg.txt
        - zytsk.txt
        - mykbe.txt
        - hbuzi.txt
        
7:
    access: loop_parallel
    targets:
        - zytsk.txt
        - hbuzi.txt
        - mykbe.txt
        - hrtqr.txt
        - cyrsk.txt
        - ngwrj.txt
2:
    access: loop_parallel
    targets:
        - rndva.txt
        - ngwrj.txt
        - jhdeg.txt
        - zytsk.txt
        - mykbe.txt
        - hbuzi.txt
9:
    access: loop_parallel
    targets:
        - zytsk.txt
        - hbuzi.txt
        - mykbe.txt
        - hrtqr.txt
        - cyrsk.txt
        - ngwrj.txt
    """
    )
