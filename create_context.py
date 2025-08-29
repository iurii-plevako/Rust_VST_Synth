import os
import datetime

def should_skip_directory(dir_name):
    skip_dirs = {'build', '.gradle', '.idea', 'out', 'target', 'bin'}
    return dir_name in skip_dirs

def should_skip_file(file_name):
    # Skip binary and temporary files, and Cargo.lock
    skip_extensions = {
        '.class', '.jar', '.war', '.ear', '.zip', '.tar', '.gz',
        '.pyc', '.pyo', '.pyd', '.dll', '.so', '.dylib',
        '.log', '.tmp', '.swp', '.DS_Store'
    }
    skip_files = {'Cargo.lock'}  # Add Cargo.lock to skip list
    return any(file_name.endswith(ext) for ext in skip_extensions) or file_name in skip_files

def collect_code(start_path):
    output = []

    # Add timestamp
    output.append(f"Code consolidated on: {datetime.datetime.now()}\n")
    output.append("Project structure and contents:\n")
    output.append("=" * 80 + "\n\n")

    for root, dirs, files in os.walk(start_path):
        # Remove excluded directories
        dirs[:] = [d for d in dirs if not should_skip_directory(d)]

        # Skip if we're in an excluded directory
        if any(skip_dir in root.split(os.sep) for skip_dir in ['.git', 'build', '.gradle', '.idea', 'vendor']):
            continue

        for file in sorted(files):
            if should_skip_file(file):
                continue

            file_path = os.path.join(root, file)
            relative_path = os.path.relpath(file_path, start_path)

            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                    # Add file path and separator
                    output.append(f"File: {relative_path}")
                    output.append("=" * 80)
                    output.append(content)
                    output.append("\n" + "=" * 80 + "\n\n")
            except Exception as e:
                output.append(f"Error reading {relative_path}: {str(e)}\n")

    return "\n".join(output)

def main():
    # Get the current directory
    current_dir = os.getcwd()

    # Generate the output
    output = collect_code(current_dir)

    # Write to file
    output_file = "project_code_consolidated.txt"
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(output)

    print(f"Code has been consolidated into {output_file}")

if __name__ == "__main__":
    main()