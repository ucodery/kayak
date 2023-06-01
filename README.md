# Kayak
You ought to know about your artifact's key-data.

Easily look up published Python packages' metadata.

## Example

Find out the same information as going to https://pypi.org/project/requests
```
$ kayak requests
Name: requests
Summary: Python HTTP for Humans.
License: Apache 2.0
Author: me@kennethreitz.org
Versions: 0.0.1, 0.2.0, 0.2.1, 0.2.2, 0.2.3, 0.2.4, 0.3.0, 0.3.1, 0.3.2, 0.3.3, 0.3.4, 0.4.0, 0.4.1, 0.5.0, 0.5.1, 0.6.0, 0.6.1, 0.6.2, 0.6.3, 0.6.4, 0.6.5, 0.6.6, 0.7.0, 0.7
7.2, 0.7.3, 0.7.4, 0.7.5, 0.7.6, 0.8.0, 0.8.1, 0.8.2, 0.8.3, 0.8.4, 0.8.5, 0.8.6, 0.8.7, 0.8.8, 0.8.9, 0.9.0, 0.9.1, 0.9.2, 0.9.3, 0.10.0, 0.10.1, 0.10.2, 0.10.3, 0.10.4, 0.1
.10.7, 0.10.8, 0.11.1, 0.11.2, 0.12.0, 0.12.1, 0.12.1, 0.13.0, 0.13.1, 0.13.2, 0.13.3, 0.13.4, 0.13.5, 0.13.6, 0.13.7, 0.13.8, 0.13.9, 0.14.0, 0.14.1, 0.14.2, 1.0.0, 1.0.1, 1
1.0.3, 1.0.4, 1.1.0, 1.2.0, 1.2.1, 1.2.2, 1.2.3, 2.0.0, 2.0.1, 2.1.0, 2.2.0, 2.2.1, 2.3.0, 2.4.0, 2.4.1, 2.4.2, 2.4.3, 2.5.0, 2.5.1, 2.5.2, 2.5.3, 2.6.0, 2.6.1, 2.6.2, 2.7.0,
, 2.8.1, 2.9.0, 2.9.1, 2.9.2, 2.10.0, 2.11.0, 2.11.1, 2.12.0, 2.12.1, 2.12.2, 2.12.3, 2.12.4, 2.12.5, 2.13.0, 2.14.0, 2.14.1, 2.14.2, 2.15.0, 2.15.1, 2.16.0, 2.16.1, 2.16.2, 
, 2.16.4, 2.16.5, 2.17.0, 2.17.1, 2.17.2, 2.17.3, 2.18.0, 2.18.1, 2.18.2, 2.18.3, 2.18.4, 2.19.0, 2.19.1, 2.20.0, 2.20.1, 2.21.0, 2.22.0, 2.23.0, 2.24.0, 2.25.0, 2.25.1, 2.26
27.0, 2.27.1, 2.28.0, 2.28.1, 2.28.2, 2.29.0, 2.30.0
Keywords: 
Classifiers: Development Status :: 5 - Production/Stable,
             Environment :: Web Environment,
             Intended Audience :: Developers,
             License :: OSI Approved :: Apache Software License,
             Natural Language :: English,
             Operating System :: OS Independent,
             Programming Language :: Python,
             Programming Language :: Python :: 3,
             Programming Language :: Python :: 3 :: Only,
             Programming Language :: Python :: 3.10,
             Programming Language :: Python :: 3.11,
             Programming Language :: Python :: 3.7,
             Programming Language :: Python :: 3.8,
             Programming Language :: Python :: 3.9,
             Programming Language :: Python :: Implementation :: CPython,
             Programming Language :: Python :: Implementation :: PyPy,
             Topic :: Internet :: WWW/HTTP,
             Topic :: Software Development :: Libraries
Links:
       Package Index: https://pypi.org/project/requests/
       Documentation: https://requests.readthedocs.io
       Source: https://github.com/psf/requests
       Homepage: https://requests.readthedocs.io
```

Dig into per-release details
```
$ kayak requests 2.30
Name: requests
Version: 2.30.0
Yanked: No
Distribution types: bdist_wheel, sdist
Wheel targets: py3
Requires Python: >=3.7
Depends On: charset-normalizer (<4,>=2) idna (<4,>=2.5) urllib3 (<3,>=1.21.1) certifi (>=2017.4.17) PySocks (!=1.5.7,>=1.5.6) ; extra == 'socks' chardet (<6,>=3.0.2) ; extra 
e_chardet_on_py3'
```
