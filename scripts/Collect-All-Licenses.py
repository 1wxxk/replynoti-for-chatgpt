from pathlib import Path
import subprocess,json,re,base64
r=Path(__file__).resolve().parents[1]
cargo="cargo"
pkgs={}
for cwd in (r,r/"settings-ui/src-tauri"):
 m=json.loads(subprocess.check_output([cargo,"metadata","--locked","--format-version","1","--filter-platform","x86_64-pc-windows-msvc"],cwd=cwd))
 ids={n["id"] for n in m["resolve"]["nodes"]}
 for p in m["packages"]:
  if p["id"] in ids and (p.get("source") or "").startswith("registry+"):pkgs[(p["name"],p["version"])]=p
notices=r/"THIRD_PARTY_NOTICES.txt"
old=notices.read_text(encoding="utf-8-sig")
npm=old[old.index("===== Settings UI npm runtime dependencies ====="):]
lines=["Third-party notices for ReplyNoti Windows x64.","Rust dependencies for both executables, including build-time packages.","Sources are unmodified crates; exact source archives are linked below."]
for (name,ver),p in sorted(pkgs.items()):
 folder=Path(p["manifest_path"]).parent
 files=[f for f in folder.iterdir() if f.is_file() and re.match(r"(?i)^(license|copying|notice|copyright)(?:[._-]|$)",f.name)]
 if p.get("license_file"):
  f=folder/p["license_file"]
  if f.is_file() and f not in files:files.append(f)
 lines.extend(["","="*72,f"{name} {ver} | {p.get('license')}",f"https://crates.io/crates/{name}/{ver}",f"https://static.crates.io/crates/{name}/{name}-{ver}.crate"])
 if files:
  for f in sorted(files):lines.extend([f"--- {f.name} ---",f.read_text(encoding="utf-8",errors="replace")])
 else:
  vcs=json.loads((folder/".cargo_vcs_info.json").read_text())
  sha=vcs["git"]["sha1"];repo=p["repository"].rstrip("/").split("github.com/")[-1]
  sources=[]
  if name=='selectors':
   # MPL notices are in the source headers; include the standard license text and exact source.
   sources=['repos/spdx/license-list-data/contents/text/MPL-2.0.txt?ref=v3.26.0']
   lines.append(f'Upstream source: https://github.com/{repo}/tree/{sha}/{vcs["path_in_vcs"]}')
  elif name=='tauri-plugin':
   sources=[f'repos/{repo}/contents/{f}?ref={sha}' for f in ('LICENSE-MIT','LICENSE-APACHE-2.0')]
  else: sources=[f'repos/{repo}/license?ref={sha}']
  for source in sources:
   raw=json.loads(subprocess.check_output(['gh','api',source]))
   license_text=base64.b64decode(raw['content']).decode('utf-8')
   if not license_text.strip(): raise RuntimeError(f'empty license {name}')
   lines.extend(['--- Upstream license ---',raw['html_url'],license_text])
   print('Upstream',name,ver,raw['html_url'])
notices.write_text("\n".join(lines)+"\n\n"+npm,encoding="utf-8")
print("Collected",len(pkgs),"Rust packages, preserved npm notices")
