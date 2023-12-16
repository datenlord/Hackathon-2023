import toml
import os

SAAF_DIR= os.path.dirname(os.path.abspath(__file__))
ROOT_DIR=os.path.join(SAAF_DIR, '../..')
TOML=os.path.join(ROOT_DIR, 'Cargo.toml')

def read_cargo_toml(file_path):
    try:
        with open(file_path, "r") as toml_file:
            cargo_toml_data = toml.load(toml_file)
            return cargo_toml_data
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return None

def get_project_name(cargo_toml_data):
    try:
        project_name = cargo_toml_data["package"]["name"]
        return project_name
    except KeyError:
        print("Error: 'name' field not found in 'package' section.")
        return None

if __name__ == "__main__":
    # Specify the path to your Cargo.toml file
    cargo_toml_path = TOML

    # Read Cargo.toml file
    cargo_toml_data = read_cargo_toml(cargo_toml_path)

    if cargo_toml_data:
        # Get and print the project name
        project_name = get_project_name(cargo_toml_data)
        if project_name:
            print(f"{project_name}")
