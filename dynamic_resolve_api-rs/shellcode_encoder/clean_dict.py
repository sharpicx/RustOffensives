with open("dictionary.txt", "r") as f:
    words = [line.strip() for line in f.readlines() if line.strip()]

seen = set()
unique_words = []
for w in words:
    if w not in seen:
        unique_words.append(w)
        seen.add(w)

if len(unique_words) < 256:
    for i in range(len(unique_words), 256):
        unique_words.append(f"tambahan_{i}")
else:
    unique_words = unique_words[:256]

with open("dictionary.txt", "w") as f:
    for w in unique_words:
        f.write(f"{w}\n")
