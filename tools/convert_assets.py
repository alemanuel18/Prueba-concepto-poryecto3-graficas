"""Extract textured COLLADA meshes to compact CPU-renderer input.

The source ZIPs remain in assets/. Run: python3 tools/convert_assets.py
Each vertex stores position and RGB sampled from its diffuse texture.
"""
from __future__ import annotations
import glob
import io
import os
import struct
import zipfile
from lxml import etree as ET
from PIL import Image

SPECS = [
    ('ring', 'Objects - Launch Star.zip', 'Launch Star/SuperSpinDriver.dae'),
    ('yoshi', 'Yoshi Star Starting Planet.zip', 'Start Planet/YoshiCaughtPlanet.dae'),
    ('fluffy', 'Fluffy Bluff Valley Planet.zip', 'CloudAcrossValleyPlanet.dae'),
    ('ice', 'Freezeflame Mountain Planet.zip', 'Mountain Planet/IceMountainPlanet.dae'),
    ('lava', 'Freezeflame Volcano Sandwich Planet.zip', 'Volcano Sandwich Planet/SandwitchVolcanoPlanet.dae'),
    ('fortress', 'World 6 Galaxies - Throwback Galaxy.zip', 'Throwback Galaxy/OldFortressPlanet.dae'),
    ('yoshi_low', 'Mini Yoshi Star Galaxy.zip', 'MiniYosshiHomeGalaxy.dae'),
    ('fluffy_low', 'Mini Fluffy Buff Galaxy.zip', 'MiniMokumokuValleyGalaxy.dae'),
    ('ice_low', 'Mini Freezeflame Galaxy.zip', 'Freezeflame Galaxy/MiniIceVolcanoGalaxy.dae'),
    ('fortress_low', 'Mini Throwback Galaxy.zip', 'MiniLongForCastleGalaxy.dae'),
    ('bush', 'Common Map Objects - Bush.zip', 'Bush/CutBush.dae'),
    ('flower', 'Common Map Objects - Flowers.zip', 'Flowers/Flower.dae'),
    ('pipe', 'Common Map Objects - Warp Pipe.zip', 'Warp Pipe/EarthenPipe.dae'),
    ('fountain', 'Common Map Objects - Water Fountain.zip', 'Water Fountain/FountainBig.dae'),
    ('geyser', 'Common Map Objects - Lava Geyser.zip', 'Lava Geyser/LavaGeyser.dae'),
    ('crate', 'Common Map Objects - Strong Block.zip', 'Strong Block/StrongBlock.dae'),
    ('falling_crate', 'Common Map Objects - Falling Crate.zip', 'Falling Crate/GravityFallBlock.dae'),
]

def tag(node, name):
    return node.find('{*}' + name)

def children(node, name):
    return node.findall('{*}' + name)

def xyz_array(src):
    a = tag(src, 'float_array')
    if a is None: return []
    floats = [float(x) for x in a.text.split()]
    accessor = src.find('.//{*}accessor')
    stride = int(accessor.get('stride','3')) if accessor is not None else 3
    return [floats[i:i+stride] for i in range(0,len(floats),stride)]

SKIES = [
    ('galaxy', 'Skyboxes - Galaxy Sky.zip', 'GalaxySkyGalaxy.png'),
    ('yoshi_sky', 'Skyboxes - Galaxy Sky.zip', 'GalaxySkyUniverse.png'),
    ('fluffy_sky', 'Skyboxes - Cloudy Sky.zip', 'PlanetCloud03.png'),
    ('ice_sky', 'Skyboxes - Aurora Sky.zip', 'AuroraStar.png'),
    ('lava_sky', 'Skyboxes - Melty Molten Sky.zip', 'PlanetGalaxy.png'),
    ('fortress_sky', 'Skyboxes - Comet Observatory Dome Skies.zip', 'AstroDomeUniverse.png'),
]

def export_skies():
    for name,key,file in SKIES:
        paths=glob.glob('assets/*'+key)
        if not paths:continue
        with zipfile.ZipFile(paths[0]) as z:
            member=next(n for n in z.namelist() if n.endswith(file))
            im=Image.open(io.BytesIO(z.read(member))).convert('RGB').resize((256,256))
            with open('assets/processed/'+name+'.sky','wb') as f:f.write(im.tobytes())

def main():
    export_skies()
    for name, key, member in SPECS:
        paths = glob.glob('assets/*' + key)
        if not paths: print('missing',name);continue
        with zipfile.ZipFile(paths[0]) as z:
            root = ET.fromstring(z.read(member))
            index = {e.get('id'):e for e in root.iter() if e.get('id')}
            up = root.find('.//{*}up_axis')
            z_up = up is not None and up.text == 'Z_UP'
            images = {}
            for e in root.findall('.//{*}image'):
                ref=tag(e,'init_from')
                if ref is None or not ref.text:continue
                path=next((f for f in z.namelist() if f.endswith('/'+ref.text) or f==ref.text),None)
                if path:
                    try: images[e.get('id')]=Image.open(io.BytesIO(z.read(path))).convert('RGB')
                    except:pass
            def material_image(mat_id):
                mat=index.get(mat_id)
                if mat is None:return None
                inst=tag(mat,'instance_effect')
                if inst is None:return None
                effect=index.get(inst.get('url','').lstrip('#'))
                if effect is None:return None
                tex=effect.find('.//{*}diffuse/{*}texture')
                if tex is None:return None
                sampler=effect.find('.//{*}newparam[@sid="'+tex.get('texture')+'"]/{*}sampler2D/{*}source')
                if sampler is None:return None
                surface=effect.find('.//{*}newparam[@sid="'+sampler.text+'"]/{*}surface/{*}init_from')
                return images.get(surface.text) if surface is not None else None
            chunks=[]
            for geometry in root.findall('.//{*}library_geometries/{*}geometry'):
                mesh=tag(geometry,'mesh')
                if mesh is None:continue
                sources={s.get('id'):xyz_array(s) for s in children(mesh,'source')}
                vertices={v.get('id'):i.get('source','').lstrip('#') for v in children(mesh,'vertices') for i in children(v,'input') if i.get('semantic')=='POSITION'}
                geo_id=geometry.get('id')
                inst=root.find('.//{*}instance_geometry[@url="#'+geo_id+'"]')
                mat_bind={m.get('symbol'):m.get('target','').lstrip('#') for m in inst.findall('.//{*}instance_material')} if inst is not None else {}
                for prim in list(mesh):
                    kind=ET.QName(prim).localname
                    if kind not in ('triangles','polylist'):continue
                    inputs=[(i.get('semantic'),i.get('source','').lstrip('#'),int(i.get('offset','0'))) for i in children(prim,'input')]
                    stride=max(x[2] for x in inputs)+1
                    vals=[int(x) for x in tag(prim,'p').text.split()]
                    verts=[vals[i:i+stride] for i in range(0,len(vals),stride)]
                    if kind=='triangles': faces=[verts[i:i+3] for i in range(0,len(verts),3)]
                    else:
                        faces=[];cursor=0
                        for count in [int(x) for x in tag(prim,'vcount').text.split()]:
                            poly=verts[cursor:cursor+count];cursor+=count
                            faces.extend([[poly[0],poly[j],poly[j+1]] for j in range(1,count-1)])
                    img=material_image(mat_bind.get(prim.get('material'),'') )
                    for face in faces:
                        tri=[]
                        for v in face:
                            attrs={s:(src,v[offset]) for s,src,offset in inputs}
                            pos_src,pos_i=attrs.get('POSITION',attrs.get('VERTEX',('',0)))
                            if pos_src in vertices:pos_src=vertices[pos_src]
                            if pos_src not in sources or pos_i>=len(sources[pos_src]):break
                            p=sources[pos_src][pos_i][:3]
                            if z_up:p=[p[0],p[2],-p[1]]
                            rgb=(180,180,190)
                            uv=attrs.get('TEXCOORD')
                            if img is not None and uv and uv[0] in sources and uv[1]<len(sources[uv[0]]):
                                u,vv=sources[uv[0]][uv[1]][:2]
                                rgb=img.getpixel((int(u*img.width)%img.width,int((1-vv)*img.height)%img.height))
                            tri.append((p,rgb))
                        if len(tri)==3:chunks.extend(tri)
            if not chunks: print('empty',name);continue
            points=[p for p,_ in chunks]
            mins=[min(p[i] for p in points) for i in range(3)]
            maxs=[max(p[i] for p in points) for i in range(3)]
            center=[(mins[i]+maxs[i])/2 for i in range(3)]
            radius=max(((sum((p[i]-center[i])**2 for i in range(3)))**.5 for p in points))
            out='assets/processed/'+name+'.mesh'
            with open(out,'wb') as f:
                f.write(struct.pack('<I',len(chunks)))
                for p,rgb in chunks:
                    f.write(struct.pack('<fffBBB',*((p[i]-center[i])/radius for i in range(3)),*rgb))
            print(name,len(chunks)//3,'triangles',os.path.getsize(out)//1024,'KiB')
if __name__=='__main__':main()
