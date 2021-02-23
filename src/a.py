cadena = ""
with open("parsear.txt") as file:
    for line in file.readlines():
        cadena+=line.strip("\n").strip("    ")
        cadena+="\\n"
print(cadena)