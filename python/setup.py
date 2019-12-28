from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
      name="ronda",
      version="1.0",
      rust_extensions=[RustExtension("ronda._ronda",
                                     binding=Binding.RustCPython)],
      packages=["ronda"],
      # rust extensions are not zip safe, just like C-extensions.
      zip_safe=False,
)