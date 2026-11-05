import importlib.util
from pathlib import Path
import unittest
import tempfile
import json
import struct

ROOT = Path(__file__).resolve().parents[1]

def module(name, path):
    spec=importlib.util.spec_from_file_location(name,ROOT/path)
    value=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value

catalog=module('research_catalog','scripts/research_catalog.py')
extract=module('research_extract','scripts/research_extract.py')

class CatalogSafety(unittest.TestCase):
    def test_only_explicit_direct_driver_links_are_eligible(self):
        base={'name':'Example','device':'keyboard','type':'driver','url':'https://cdn.shopify.com/file.exe?v=1'}
        self.assertEqual(catalog.classify(base),'eligible_direct_windows_package')
        for changes,expected in [
            ({'type':'firmware'},'excluded_firmware_only'),
            ({'type':'web_driver'},'excluded_browser_configurator'),
            ({'url':'https://www.mediafire.com/file/driver.exe/file'},'excluded_hosted_landing_page'),
            ({'url':'https://unknown.invalid/file.exe'},'excluded_unreviewed_download_host'),
            ({'url':'http://cdn.shopify.com/file.exe'},'excluded_non_https')]:
            self.assertEqual(catalog.classify({**base,**changes}),expected)
    def test_product_links_and_unrelated_cards_do_not_become_downloads(self):
        parser=catalog.CatalogParser()
        parser.feed('<article><h3>Not a driver</h3></article><article data-driver-card data-driver-device="keyboard" data-driver-type="driver"><h3>F &amp; X</h3><a href="/products/example">Product</a><a data-driver-download href="https://cdn.shopify.com/file.exe?v=1&amp;x=2">Windows</a></article>')
        self.assertEqual(len(parser.cards),1)
        self.assertEqual(parser.cards[0]['name'],'F & X')
        self.assertEqual(parser.cards[0]['url'],'https://cdn.shopify.com/file.exe?v=1&x=2')
        self.assertEqual(parser.cards[0]['product_url'],'https://www.aulagaming.com/products/example')
    def test_extraction_rejects_traversal_drive_and_network_names(self):
        for name in ['../escape','app/../../escape','/absolute','C:\\Windows\\file','\\\\server\\share','app/\x00secret']:
            with self.assertRaises(ValueError,msg=name):extract.safe_name(name)
        self.assertEqual(str(extract.safe_name('app/Dev/kb/KB.ini')),'app/Dev/kb/KB.ini')
    def test_cached_artifact_without_matching_provenance_is_preserved_and_rejected(self):
        (ROOT/'target').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT/'target') as directory:
            out=Path(directory); artifact=out/'downloads/example.exe'
            artifact.parent.mkdir(); artifact.write_bytes(b'MZ-original-download')
            card={'name':'Example','url':'https://cdn.shopify.com/file.exe'}
            with self.assertRaisesRegex(ValueError,'provenance'):
                catalog.download(card,out,1024)
            artifact.with_suffix('.exe.metadata.json').write_text(json.dumps({'source_url':card['url'],'sha256':'0'*64,'bytes':artifact.stat().st_size}))
            with self.assertRaisesRegex(ValueError,'provenance mismatch'):
                catalog.download(card,out,1024)
            self.assertEqual(artifact.read_bytes(),b'MZ-original-download')
    def test_hid_ampersands_are_parsed_as_data_not_xml_entities(self):
        parser=extract.DeviceModes()
        parser.feed('<config><mode vid="0C45" pid="800A" hid_interface="VID_0C45&PID_800A&MI_00"/></config>')
        self.assertEqual(parser.modes,[{'element':'mode','vid':'0C45','pid':'800A','hid_interface':'VID_0C45&PID_800A&MI_00'}])

class AdvancedInstallerSafety(unittest.TestCase):
    @staticmethod
    def fixture(name='MainFeature.cab'):
        data=b'MSCF'+b'\0'*508+b'payload remains unchanged'
        carrier=b'MZ'+b'\0'*62+bytes(v^255 for v in data[:512])+data[512:]
        record=struct.pack('<5I',7,2,len(data),64,len(name))+name.encode('utf-16-le')
        return carrier+record+b'ADVINSTSFX',data
    def test_only_prefix512_is_decoded_and_metadata_is_bound(self):
        carrier,expected=self.fixture()
        decoded,record=extract.advanced_container(carrier,'MainFeature.cab')
        self.assertEqual(decoded,expected)
        self.assertEqual(record['offset'],64)
        self.assertEqual(record['bytes'],len(expected))
        self.assertEqual(record['encoding'],'first512_bytes_complement_ff')
    def test_bad_bounds_encoding_and_unknown_names_are_rejected(self):
        carrier,_=self.fixture()
        for invalid in [carrier[:-1],b'ZZ'+carrier[2:]]:
            with self.assertRaises(ValueError):extract.advanced_container(invalid,'MainFeature.cab')
        with self.assertRaises(ValueError):extract.advanced_container(carrier,'../outside.cab')
        index=carrier.rfind('MainFeature.cab'.encode('utf-16-le'))
        for field,value in [(1,3),(2,0xffffffff),(3,0xffffff00),(4,99)]:
            bad=bytearray(carrier);struct.pack_into('<I',bad,index-20+field*4,value)
            with self.assertRaises(ValueError):extract.advanced_container(bytes(bad),'MainFeature.cab')

if __name__=='__main__':unittest.main()
