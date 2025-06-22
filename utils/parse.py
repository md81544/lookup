#!/usr/bin/env python3

import json


def extract_word_defs(json_file):
    try:
        with open(json_file, "r", encoding="utf-8") as f:
            data = json.load(f)

        results = []

        for entry in data.values():
            word = entry.get("word")
            meanings = entry.get("meanings", [])

            if word and meanings:
                for meaning in meanings:
                    definition = meaning.get("def")
                    if definition:
                        print(f"{word.lower()}|{definition[0].upper() + definition[1:]}")

        return results

    except Exception as e:
        print(f"Error processing file: {str(e)}")
        return []


def main():
    extract_word_defs("/Users/mcd/tmp/allwords_wordset.json")


if __name__ == "__main__":
    main()
