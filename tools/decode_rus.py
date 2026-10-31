import base64, os, json, sys
K=b"#*8&@45***453$"
def d(b): return bytes((c^K[i%len(K)])^(i&255) for i,c in enumerate(b))
def dec(s):
    s=s.replace("_","/")
    s+="="*(-len(s)%4)
    return d(base64.b64decode(d(base64.b64decode(s)))).decode()
src="apk_assets/assets/headsetRusConfig"; out={}
for f in sorted(os.listdir(src)):
    try: n=dec(f)
    except Exception as e: n="?"+f
    j=json.load(open(f"{src}/{f}"))
    out[n]=j
    a=j.get("androidConfig",{})
    print(n,"|",a.get("sdkVendor"),a.get("modelId"),a.get("sppUUIDType"))
json.dump(out,open("rus_configs.json","w"),indent=1)
