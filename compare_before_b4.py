import json, os

def parse_criterion_home(home):
    points = {}
    for group in sorted(os.listdir(home)):
        gdir = os.path.join(home, group)
        if not os.path.isdir(gdir) or group in ("report",):
            continue
        for lane in sorted(os.listdir(gdir)):
            ldir = os.path.join(gdir, lane)
            if not os.path.isdir(ldir):
                continue
            for param in sorted(os.listdir(ldir)):
                est_path = os.path.join(ldir, param, "new", "estimates.json")
                if not os.path.exists(est_path):
                    continue
                with open(est_path) as f:
                    est = json.load(f)
                median = est["median"]
                mean = est["mean"]
                points[f"{group}/{lane}/{param}"] = {
                    "median_ns": median["point_estimate"],
                    "median_ci": median["confidence_interval"],
                    "mean_ns": mean["point_estimate"],
                    "mean_ci": mean["confidence_interval"],
                    "std_dev_ns": est["std_dev"]["point_estimate"],
                }
    return points

b4_points = parse_criterion_home("/app/b4-results/config-scalar")
b5_before = parse_criterion_home("/app/b5-results/before")

print(f"{'Point':55s} | {'B4 config-scalar':>16s} | {'B5 same-day before':>18s} | {'Drift %':>9s}")
print("-" * 105)
for k in sorted(b4_points.keys()):
    b4 = b4_points[k]
    b5 = b5_before.get(k)
    b4_ms = b4["median_ns"] / 1e6
    if b5:
        b5_ms = b5["median_ns"] / 1e6
        drift = ((b5_ms - b4_ms) / b4_ms) * 100
        print(f"{k:55s} | {b4_ms:13.3f} ms | {b5_ms:15.3f} ms | {drift:+8.2f}%")
    else:
        print(f"{k:55s} | {b4_ms:13.3f} ms | {'MISSING':>18s} |")
