"""
Options:
- fetch
- apply
- edit
- list: list configs available for this platform
"""

__version__ = "1.0.0"

import os
import sys
import yaml
import shutil
import platform
from datetime import datetime

difftool = "git --no-pager diff --no-index"
configs_dir = os.path.abspath(os.path.expanduser("~/.confmgr"))
backup_dir = "backup"


def _get_platform() -> str:
    system_name = platform.system()
    if system_name == "Linux":
        return "linux"
    elif system_name == "Windows":
        return "windows"
    elif system_name == "Darwin":
        return "macos"
    else:
        sys.exit("platform {} is not supported".format(system_name))


def _load_yaml(file_path) -> dict:
    with open(file_path, "r") as f:
        try:
            return yaml.safe_load(f)
        except yaml.YAMLError as e:
            sys.exit("invalid yaml file:\n{}".format(e))


def _get_entries() -> dict:

    # load the configs file
    # TODO check existence
    configs = _load_yaml(os.path.join(configs_dir, "confmgr.yaml"))

    time_string = datetime.now().strftime(r"%Y%m%d%H%M%S")

    entries = {}
    for label, config in configs.items():
        if not config.get("ignore"):
            path_dest = config.get(_get_platform())
            if path_dest:
                path_source = config.get("source")
                if path_source:
                    entries[label] = {
                        "src": os.path.abspath(os.path.join(configs_dir, path_source)),
                        "src_rel": os.path.relpath(os.path.join(configs_dir, path_source)),
                        "dest": os.path.abspath(os.path.expanduser(path_dest)),
                        "backup": os.path.abspath(os.path.join(backup_dir, time_string, path_source)),
                    }
                else:
                    sys.exit("no source specified for {}".format(label))
    return entries


def _iter_labels(labels):
    entries = _get_entries()
    if len(labels) < 1:
        yield from entries.items()
    else:
        for label in labels:
            entry = entries.get(label)
            if entry:
                yield label, entry
            else:
                print("label {} is invalid\n".format(label))


def _iter_labels_safe(labels, need_src=False, need_dest=False, show_errors=True):
    for label, entry in _iter_labels(labels):
        err = ""
        if need_src and not os.path.isfile(entry["src"]):
            err += "source file {} not found\n".format(entry["src_rel"])
        if need_dest and not os.path.isfile(entry["dest"]):
            err += "destination file {} not found\n".format(entry["dest"])
        if not err:
            yield label, entry
        elif show_errors:
            print(err)


def fetch(labels):
    for label, entry in _iter_labels_safe(labels, need_dest=True):
        print("fetching {} ({} -> {})".format(label, entry["dest"], entry["src_rel"]))
        os.makedirs(os.path.dirname(entry["src"]), exist_ok=True)
        shutil.copyfile(entry["dest"], entry["src"])


def apply(labels):
    for label, entry in _iter_labels_safe(labels, need_src=True, need_dest=True):
        print("applying {} ({} -> {})".format(label, entry["src_rel"], entry["dest"]))
        os.makedirs(os.path.dirname(entry["backup"]), exist_ok=True)
        shutil.copyfile(entry["dest"], entry["backup"])
        shutil.copyfile(entry["src"], entry["dest"])


def edit(labels):
    for label, entry in _iter_labels_safe(labels, need_src=True):
        print("opening {} ({})".format(label, entry["src_rel"]))
        if _get_platform() == "windows":
            os.system(entry["src"])
        else:
            raise NotImplementedError


def diff(labels):
    for label, entry in _iter_labels_safe(labels, need_src=True, need_dest=True):
        print("showing diff for {}".format(label))
        os.system('{} "{}" "{}"'.format(difftool, entry["dest"], entry["src_rel"]))
        print("")


def ignore(labels):
    # TODO
    pass


def main():

    # parse arguments
    if len(sys.argv) < 2:
        mode = None
    else:
        _, mode, *args = sys.argv

    if mode == "fetch":
        fetch(args)
    elif mode == "apply":
        apply(args)
    elif mode == "edit":
        edit(args)
    elif mode == "diff":
        diff(args)
    elif mode == "ignore":
        ignore(args)

    elif mode == "list":
        print(", ".join(list(_get_entries().keys())))
    else:
        print(__doc__, end="")


if __name__ == "__main__":
    main()
