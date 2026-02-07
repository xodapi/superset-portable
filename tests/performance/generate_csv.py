import csv
import time
import os
import random

FILE_PATH = "giant_file.csv"
ROWS = 1_000_000

print(f"Generating {ROWS} rows to {FILE_PATH}...")
start = time.time()

with open(FILE_PATH, 'w', newline='', encoding='utf-8') as f:
    writer = csv.writer(f)
    writer.writerow(["id", "name", "value", "category", "date", "description"])
    
    # Batch write for speed
    batch_size = 10000
    batch = []
    
    for i in range(ROWS):
        batch.append([
            i, 
            f"Item {i}", 
            random.random() * 1000, 
            random.choice(["A", "B", "C", "D"]), 
            "2024-01-01", 
            "This is a long description text to fill up space and make the file larger."
        ])
        
        if len(batch) >= batch_size:
            writer.writerows(batch)
            batch = []
    
    if batch:
        writer.writerows(batch)

end = time.time()
size_mb = os.path.getsize(FILE_PATH) / (1024 * 1024)

print(f"Done in {end - start:.2f}s. File size: {size_mb:.2f} MB")
