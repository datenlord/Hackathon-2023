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




PRJ_NAME=stdout("python3 scripts/saaf/project_name.py")
print("PRJ_NAME",PRJ_NAME)

# get the binary file

stdout(f"cp target/release/{PRJ_NAME} docker/SysNode/{PRJ_NAME}")

# generate entrypoint.sh

entrypoint_tmp = f"""
#!/bin/bash

echo "Node id: $SYS_NODEID"
echo "Who am i: $(whoami)"

# tc qdisc add dev eth0 root netem delay 100ms

# tc qdisc add dev eth0 root tbf rate 1mbit burst 10kb latency 70ms

timeout 10 ping baidu.com

cd /usr/local/bin/
ls /etc/{PRJ_NAME}/

{PRJ_NAME} $SYS_NODEID /etc/{PRJ_NAME}/files
"""

with open('docker/SysNode/entrypoint.sh', 'w') as file:
    file.write(entrypoint_tmp)


# generate Dockerfile

docker_tmp = f"""
    
FROM {PRJ_NAME}_env:v1 as {PRJ_NAME}

LABEL maintainers="ActivePeter"

COPY {PRJ_NAME} /usr/local/bin/{PRJ_NAME}

COPY entrypoint.sh /etc/{PRJ_NAME}/

RUN chmod +x /usr/local/bin/{PRJ_NAME}

RUN chmod +x /etc/{PRJ_NAME}/entrypoint.sh

ENTRYPOINT ["bash","/etc/{PRJ_NAME}/entrypoint.sh"]

"""

with open('docker/SysNode/Dockerfile', 'w') as file:
    file.write(docker_tmp)

stdout(f"docker build -t {PRJ_NAME}:v1 docker/SysNode --no-cache")

# docker build -t $PRJ_NAME:v1 docker/SysNode --no-cache

# rm -f docker/BaseEnv/Dockerfile