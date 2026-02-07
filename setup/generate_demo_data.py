# -*- coding: utf-8 -*-
"""
Генератор расширенных демо-данных РЖД для Superset Portable.
Создает реалистичные данные станций и маршрутов на основе крупных городов России.
"""

import csv
import random
import math
import json
from pathlib import Path

# Конфигурация
SCRIPT_DIR = Path(__file__).parent
DATA_DIR = SCRIPT_DIR.parent / "docs" / "demo_data"

# Список городов с координатами (широта, долгота) и регионами
CITIES = [
    # Центральный ФО
    ("Москва", 55.7558, 37.6173, "Московская область", "Московская ж/д"),
    ("Санкт-Петербург", 59.9343, 30.3351, "Ленинградская область", "Октябрьская ж/д"),
    ("Воронеж", 51.6703, 39.1899, "Воронежская область", "Юго-Восточная ж/д"),
    ("Ярославль", 57.6261, 39.8845, "Ярославская область", "Северная ж/д"),
    ("Рязань", 54.6095, 39.7126, "Рязанская область", "Московская ж/д"),
    ("Тула", 54.1931, 37.6172, "Тульская область", "Московская ж/д"),
    ("Липецк", 52.6088, 39.5992, "Липецкая область", "Юго-Восточная ж/д"),
    ("Брянск", 53.2709, 34.3643, "Брянская область", "Московская ж/д"),
    ("Тверь", 56.8596, 35.9119, "Тверская область", "Октябрьская ж/д"),
    ("Курск", 51.7240, 36.1900, "Курская область", "Московская ж/д"),
    
    # Приволжский ФО
    ("Нижний Новгород", 56.2965, 43.9361, "Нижегородская область", "Горьковская ж/д"),
    ("Казань", 55.7940, 49.1118, "Республика Татарстан", "Горьковская ж/д"),
    ("Самара", 53.1955, 50.1018, "Самарская область", "Куйбышевская ж/д"),
    ("Уфа", 54.7388, 55.9721, "Республика Башкортостан", "Куйбышевская ж/д"),
    ("Пермь", 58.0048, 56.2377, "Пермский край", "Свердловская ж/д"),
    ("Саратов", 51.5430, 46.0345, "Саратовская область", "Приволжская ж/д"),
    ("Ижевск", 56.8619, 53.2324, "Удмуртская Республика", "Горьковская ж/д"),
    ("Ульяновск", 54.3142, 48.4031, "Ульяновская область", "Куйбышевская ж/д"),
    ("Оренбург", 51.7666, 55.1005, "Оренбургская область", "Южно-Уральская ж/д"),
    ("Пенза", 53.1950, 45.0229, "Пензенская область", "Куйбышевская ж/д"),
    
    # Южный и Сев-Кав ФО
    ("Ростов-на-Дону", 47.2357, 39.7015, "Ростовская область", "Северо-Кавказская ж/д"),
    ("Краснодар", 45.0354, 38.9753, "Краснодарский край", "Северо-Кавказская ж/д"),
    ("Волгоград", 48.7072, 44.5169, "Волгоградская область", "Приволжская ж/д"),
    ("Сочи", 43.5855, 39.7231, "Краснодарский край", "Северо-Кавказская ж/д"),
    ("Астрахань", 46.3479, 48.0336, "Астраханская область", "Приволжская ж/д"),
    ("Севастополь", 44.6166, 33.5254, "Севастополь", "Крымская ж/д"),
    ("Симферополь", 44.9572, 34.1108, "Республика Крым", "Крымская ж/д"),
    ("Махачкала", 42.9831, 47.5046, "Республика Дагестан", "Северо-Кавказская ж/д"),
    ("Ставрополь", 45.0445, 41.9691, "Ставропольский край", "Северо-Кавказская ж/д"),
    
    # Уральский ФО
    ("Екатеринбург", 56.8389, 60.6057, "Свердловская область", "Свердловская ж/д"),
    ("Челябинск", 55.1600, 61.4025, "Челябинская область", "Южно-Уральская ж/д"),
    ("Тюмень", 57.1613, 65.5250, "Тюменская область", "Свердловская ж/д"),
    ("Магнитогорск", 53.4000, 58.9725, "Челябинская область", "Южно-Уральская ж/д"),
    ("Сургут", 61.2540, 73.3964, "Ханты-Мансийский АО", "Свердловская ж/д"),
    
    # Сибирский ФО
    ("Новосибирск", 55.0084, 82.9357, "Новосибирская область", "Западно-Сибирская ж/д"),
    ("Омск", 54.9885, 73.3242, "Омская область", "Западно-Сибирская ж/д"),
    ("Красноярск", 56.0153, 92.8932, "Красноярский край", "Красноярская ж/д"),
    ("Барнаул", 53.3498, 83.7836, "Алтайский край", "Западно-Сибирская ж/д"),
    ("Иркутск", 52.2869, 104.2807, "Иркутская область", "Восточно-Сибирская ж/д"),
    ("Томск", 56.5010, 84.9924, "Томская область", "Западно-Сибирская ж/д"),
    ("Кемерово", 55.3551, 86.0872, "Кемеровская область", "Западно-Сибирская ж/д"),
    ("Новокузнецк", 53.7596, 87.1216, "Кемеровская область", "Западно-Сибирская ж/д"),
    
    # Дальневосточный ФО
    ("Хабаровск", 48.4814, 135.0721, "Хабаровский край", "Дальневосточная ж/д"),
    ("Владивосток", 43.1198, 131.8869, "Приморский край", "Дальневосточная ж/д"),
    ("Улан-Удэ", 51.8348, 107.5845, "Республика Бурятия", "Восточно-Сибирская ж/д"),
    ("Чита", 52.0317, 113.5009, "Забайкальский край", "Забайкальская ж/д"),
    ("Якутск", 62.0397, 129.7422, "Республика Саха (Якутия)", "Железные дороги Якутии"),
    
    # Северо-Западный ФО
    ("Калининград", 54.7104, 20.4522, "Калининградская область", "Калининградская ж/д"),
    ("Архангельск", 64.5473, 40.5602, "Архангельская область", "Северная ж/д"),
    ("Мурманск", 68.9665, 33.0833, "Мурманская область", "Октябрьская ж/д"),
    ("Вологда", 59.2205, 39.8915, "Вологодская область", "Северная ж/д"),
    ("Петрозаводск", 61.7900, 34.3900, "Республика Карелия", "Октябрьская ж/д"),
]

def generate_stations():
    """Генерация списка станций на основе городов"""
    stations = []
    
    # Типы станций для расширения
    # 0 - Пассажирская, 1 - Товарная, 2 - Сортировочная
    
    idx = 1
    for city, lat, lon, region, branch in CITIES:
        # 1. Главный вокзал (Пассажирская)
        stations.append({
            "id": idx,
            "name": f"{city}-Главный",
            "city": city,
            "region": region,
            "latitude": lat,
            "longitude": lon,
            "passengers_day": int(random.gauss(20000, 5000) * (3.0 if city in ["Москва", "Санкт-Петербург"] else 1.0)),
            "cargo_tons_year": int(random.gauss(100000, 20000)),
            "railway_branch": branch,
            "station_class": 1
        })
        idx += 1
        
        # 2. Товарная станция (рядом, но чуть в стороне)
        if random.random() > 0.2: # 80% городов имеют товарную станцию
            lat_offset = (random.random() - 0.5) * 0.05
            lon_offset = (random.random() - 0.5) * 0.05
            stations.append({
                "id": idx,
                "name": f"{city}-Товарная",
                "city": city,
                "region": region,
                "latitude": lat + lat_offset,
                "longitude": lon + lon_offset,
                "passengers_day": int(random.gauss(500, 100)),
                "cargo_tons_year": int(random.gauss(2000000, 500000)),
                "railway_branch": branch,
                "station_class": 2
            })
            idx += 1
            
        # 3. Сортировочная (для крупных узлов)
        if random.random() > 0.6: 
            lat_offset = (random.random() - 0.5) * 0.08
            lon_offset = (random.random() - 0.5) * 0.08
            stations.append({
                "id": idx,
                "name": f"{city}-Сортировочная",
                "city": city,
                "region": region,
                "latitude": lat + lat_offset,
                "longitude": lon + lon_offset,
                "passengers_day": 0,
                "cargo_tons_year": int(random.gauss(5000000, 1000000)),
                "railway_branch": branch,
                "station_class": 3
            })
            idx += 1
            
    return stations

def calculate_distance(lat1, lon1, lat2, lon2):
    """Расчет расстояния между координатами в км"""
    R = 6371  # Радиус Земли
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    a = math.sin(dlat/2) * math.sin(dlat/2) + \
        math.cos(math.radians(lat1)) * math.cos(math.radians(lat2)) * \
        math.sin(dlon/2) * math.sin(dlon/2)
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))
    return R * c

def generate_routes(stations):
    """Генерация маршрутов между станциями"""
    routes = []
    route_id = 1
    
    # Создаем граф соседства
    # Для простоты соединяем каждый город с 3-5 ближайшими
    
    # Фильтруем только главные станции
    main_stations = [s for s in stations if "Главный" in s["name"]]
    
    for origin in main_stations:
        # Находим расстояния до всех других
        distances = []
        for dest in main_stations:
            if origin["id"] == dest["id"]:
                continue
            dist = calculate_distance(origin["latitude"], origin["longitude"], 
                                      dest["latitude"], dest["longitude"])
            distances.append((dest, dist))
            
        # Сортируем по расстоянию и берем топ-4 ближайших
        distances.sort(key=lambda x: x[1])
        nearest = distances[:4]
        
        for dest, dist in nearest:
            # Двунаправленные маршруты, но ID уникальны
            # Чтобы не дублировать A->B и B->A, добавляем только если id1 < id2
            if origin["id"] < dest["id"]:
                start_coords = [origin["longitude"], origin["latitude"]]
                end_coords = [dest["longitude"], dest["latitude"]]
                
                routes.append({
                    "id": route_id,
                    "origin_id": origin["id"],
                    "origin_name": origin["name"],
                    "dest_id": dest["id"],
                    "dest_name": dest["name"],
                    "distance_km": round(dist, 1),
                    "trains_per_day": random.randint(2, 20),
                    # GeoJSON geometry for Deck.gl
                    "geometry": json.dumps({
                        "type": "LineString",
                        "coordinates": [start_coords, end_coords]
                    })
                })
                route_id += 1
                
    return routes

def generate_geojson_lines(routes, stations):
    """Генерация GeoJSON линий на основе маршрутов"""
    station_map = {s["id"]: (s["longitude"], s["latitude"]) for s in stations}
    
    features = []
    for r in routes:
        if r["origin_id"] not in station_map or r["dest_id"] not in station_map:
            continue
            
        start_coords = station_map[r["origin_id"]]
        end_coords = station_map[r["dest_id"]]
        
        # Генерация метрик для линии
        load_percent = random.randint(30, 95)
        capacity = random.choice([50, 80, 100, 120]) # млн тонн
        
        feature = {
            "type": "Feature",
            "geometry": {
                "type": "LineString",
                "coordinates": [start_coords, end_coords]
            },
            "properties": {
                "id": r["id"],
                "origin": r["origin_name"],
                "destination": r["dest_name"],
                "distance_km": r["distance_km"],
                "trains_per_day": r["trains_per_day"],
                "load_percent": load_percent,
                "capacity_mln_tons": capacity,
                "status": "Critical" if load_percent > 85 else "Normal"
            }
        }
        features.append(feature)
        
    return {
        "type": "FeatureCollection",
        "features": features
    }

def main():
    print("Генерация демо-данных РЖД...")
    
    # Генерация станций
    stations = generate_stations()
    print(f"Сгенерировано {len(stations)} станций")
    
    # Сохранение станций
    stations_file = DATA_DIR / "rzd_stations_full.csv"
    with open(stations_file, "w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "id", "name", "city", "region", "latitude", "longitude", 
            "passengers_day", "cargo_tons_year", "railway_branch", "station_class"
        ])
        writer.writeheader()
        writer.writerows(stations)
    print(f"Сохранено: {stations_file}")
    
    # Генерация маршрутов
    routes = generate_routes(stations)
    print(f"Сгенерировано {len(routes)} маршрутов")
    
    # Сохранение маршрутов
    routes_file = DATA_DIR / "rzd_routes.csv"
    with open(routes_file, "w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "id", "origin_id", "origin_name", "dest_id", "dest_name", 
            "distance_km", "trains_per_day", "geometry"
        ])
        writer.writeheader()
        writer.writerows(routes)
    print(f"Сохранено: {routes_file}")

    # Генерация GeoJSON
    geojson = generate_geojson_lines(routes, stations)
    print(f"Сгенерировано {len(geojson['features'])} линий GeoJSON")
    
    geojson_file = DATA_DIR / "rzd_lines.geojson"
    with open(geojson_file, "w", encoding="utf-8") as f:
        json.dump(geojson, f, ensure_ascii=False, indent=2)
    print(f"Сохранено: {geojson_file}")

if __name__ == "__main__":
    random.seed(42) # Для воспроизводимости
    if not DATA_DIR.exists():
        DATA_DIR.mkdir(parents=True)
    main()
