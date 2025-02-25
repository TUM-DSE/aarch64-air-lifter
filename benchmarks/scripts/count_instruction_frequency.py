from collections import defaultdict
import os

# Counts occurrence of instructions in the binary files and writes these to a csv file

PROJECT_PATH = "tests/bin"

# {file_path: {instruction: frequency}}
frequency_dict = {}
unique_instructions = set()
files = None
for root, dirs, files in os.walk(PROJECT_PATH):
    files = files
    for file in files:
        file_path = os.path.join(root, file)
        command = f"aarch64-linux-gnu-objdump -d -S -EB -M no-aliases {file_path} -j .text | grep '^\ ' | awk '{{print $3}}' | sort | uniq -c | sort -n -r"
        result = os.popen(command).read()
        for line in result.split("\n"):
            if not line:
                continue
            frequency, instruction = line.strip().split()
            frequency = int(frequency)
            if file not in frequency_dict:
                frequency_dict[file] = defaultdict(int)
            frequency_dict[file][instruction] = frequency
            unique_instructions.add(instruction)

instructions_per_file = {file: sum(frequency_dict[file].values()) for file in files}

# write to csv
with open("frequency.csv", "w") as f:
    header = "instruction_name,"
    for file in files:
        header += file + "," + file + " instruction frequency in %,"
    f.write(header + "\n")
    for instruction in unique_instructions:
        f.write(instruction)
        for file in files:
            f.write(",")
            f.write(str(frequency_dict[file].get(instruction, 0)))
            f.write(",")
            f.write(
                str(
                    frequency_dict[file].get(instruction, 0)
                    / instructions_per_file[file]
                )
            )
        f.write("\n")

    # add a line with total instructions per file
    f.write("total")
    for file in files:
        f.write(",")
        f.write(str(instructions_per_file[file]))
