# Brainfuck interpreter

**Caracteristicas**:

- Tamaño de memoria fijo de la cita: 2^16^ = 65536
- La cinta se recorre cíclicamente
- El último argumento se establecera en 0 (*EOF*), si se vuelve a solicitar un argumento generará un error. Ej: ",[.,]," "Hola mundo"
- Tamaño de la celdas de memoria: 2^8^ = 256
- Las celdas se recorren cíclicamente
- Salida, *"String: UTF-8" \[u8\]*, si un carácter no se puede interpretar saldrá `�`
