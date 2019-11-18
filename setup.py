from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(name='libronda',
      version="0.1.0",
      rust_extensions=[RustExtension('libronda', 'Cargo.toml',  binding=Binding.PyO3)],
      test_suite="tests",
      zip_safe=False)