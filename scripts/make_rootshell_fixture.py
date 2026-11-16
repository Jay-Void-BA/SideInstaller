"""Offline, public test credentials and an IPA exercising recursive signing."""
import pathlib
import copy
import plistlib
import struct
import subprocess
import sys
import zipfile


def macho():
    data = bytearray(512)
    struct.pack_into('<8I', data, 0, 0xFEEDFACF, 0x100000C, 0, 2, 3, 160, 0, 0)
    for offset, name, fileoff, size in [(32, b'__TEXT', 0, 512), (104, b'__LINKEDIT', 512, 0)]:
        struct.pack_into('<II16sQQQQIIII', data, offset, 0x19, 72, name, fileoff, size, fileoff, size, 7, 5, 0, 0)
    struct.pack_into('<4I', data, 176, 0x1D, 16, 512, 0)
    return data


def create(folder, large_directory=False):
    folder = pathlib.Path(folder)
    folder.mkdir(parents=True, exist_ok=True)
    # This key belongs only to an offline fixture, never an Apple Account.
    subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '2',
                    '-subj', '/CN=siboot offline test only', '-keyout', str(folder / 'test-key.pem'),
                    '-out', str(folder / 'test-cert.pem')], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(['openssl', 'pkcs8', '-topk8', '-nocrypt', '-in', str(folder / 'test-key.pem'), '-outform', 'DER', '-out', str(folder / 'test-key.der')], check=True)
    subprocess.run(['openssl', 'x509', '-in', str(folder / 'test-cert.pem'), '-outform', 'DER', '-out', str(folder / 'test-cert.der')], check=True)
    with zipfile.ZipFile(folder / 'Fixture.ipa', 'w', zipfile.ZIP_DEFLATED) as archive:
        for prefix, executable, identifier in [('', 'Main', 'example.offline'), ('Extensions/Widget.appex/', 'Widget', 'example.offline.widget'), ('Frameworks/Versioned.framework/', 'Versioned', 'example.offline.framework')]:
            base = 'Payload/Fixture.app/' + prefix
            info = plistlib.dumps(dict(CFBundleIdentifier=identifier, CFBundleExecutable=executable, CFBundleName='Fixture', CFBundleVersion='1'))
            if 'framework' in prefix:
                archive.writestr(base + 'Versions/A/Versioned', macho())
                archive.writestr(base + 'Versions/A/Resources/Info.plist', info)
                for name, target in [('Versions/Current', 'A'), ('Versioned', 'Versions/Current/Versioned'), ('Resources', 'Versions/Current/Resources'), ('Info.plist', 'Resources/Info.plist')]:
                    entry = zipfile.ZipInfo(base + name)
                    entry.create_system = 3
                    entry.external_attr = 0o120777 << 16
                    archive.writestr(entry, target)
            else:
                archive.writestr(base + 'Info.plist', info)
                archive.writestr(base + executable, macho())
            archive.writestr(base + '.config', b'must be sealed')
        for name in ['Resources/.config', 'PlugIns/Tools/injected.dylib', 'Hidden/.bin/driver']:
            archive.writestr('Payload/Fixture.app/' + name, b'configuration' if name.endswith('.config') else macho())
        if large_directory:
            for index in range(320):
                archive.writestr(f'Payload/Fixture.app/ManyFiles/resource-{index:04d}.dat', b'must be sealed beyond the first readdir buffer')
            (folder / 'large-directory.fixture').write_text('320')
        link = zipfile.ZipInfo('Payload/Fixture.app/Resources/link.dylib')
        link.create_system = 3
        link.external_attr = 0o120777 << 16
        archive.writestr(link, '../PlugIns/Tools/injected.dylib')
    with zipfile.ZipFile(folder / 'Fixture.ipa') as original:
        entries = [(copy.copy(entry), original.read(entry)) for entry in original.infolist()]
    for filename, targets in [('Escape.ipa', [('a', '../../../../outside')]),
                              ('Cycle.ipa', [('a', 'b'), ('b', 'a')]),
                              ('Absolute.ipa', [('a', '/test-key.der')])]:
        with zipfile.ZipFile(folder / filename, 'w', zipfile.ZIP_DEFLATED) as archive:
            for entry, data in entries:
                archive.writestr(copy.copy(entry), data)
            for name, target in targets:
                entry = zipfile.ZipInfo('Payload/Fixture.app/' + name)
                entry.create_system = 3
                entry.external_attr = 0o120777 << 16
                archive.writestr(entry, target)


if __name__ == '__main__':
    create(sys.argv[1], '--large-directory' in sys.argv[2:])
