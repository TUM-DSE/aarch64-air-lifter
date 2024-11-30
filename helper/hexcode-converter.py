import re

# receives hex code in array format and converts it into spaced binary
def clean_hex(hexcode: str):
    # remove white and remove last commata
    hexcode = re.sub("0x|,", "", hexcode)
    return hexcode.split(" ")

def convert_endianness(hexcode: list[str]):
    return hexcode[::-1]

def convert_to_binary(hexcode: list[str]):
    return [format(int(x, 16), '08b') for x in hexcode]

def print_binary(hexcode: list[str]):
    for x in hexcode:
        print(x, end=" ")

def process_hex(hexcode: str):
    hexcode = clean_hex(hexcode)
    hexcode = convert_endianness(hexcode)
    print_binary(convert_to_binary(hexcode)) 
    print("")

def process_binary(binary: str):
    binary = re.sub("(\t)| ", "", binary)
    if len(binary) != 32:
        print("Invalid binary length")
        return
    
    for b in range(len(binary)-1,0,-8):
        b = binary[b-7:b]
        # Format as 2-digit uppercase hex and add 0x prefix
        hex_val = "0x" + format(int(b, 2), '02X')
        print(hex_val, end=", ")
    print("")