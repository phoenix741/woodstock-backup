#pragma once

#include <QSharedData>
#include "indexmanifestentry.h"

class IndexFileEntryData : public QSharedData
{
public:
  IndexFileEntryData()
      : journal(false),
        index(-1),
        read(false),
        deleted(false),
        lastModifiedDate(-1),
        size(-1) {}
  IndexFileEntryData(const IndexFileEntryData &other)
      : QSharedData(other),
        path(other.path),
        journal(other.journal),
        index(other.index),
        read(other.read),
        deleted(other.deleted),
        files(other.files),
        lastModifiedDate(other.lastModifiedDate),
        size(other.size) {}
  ~IndexFileEntryData(){};

  QString path;
  bool journal;
  qint64 index;

  bool read;
  bool deleted;
  QHash<QString, IndexFileEntry> files;

  qint64 lastModifiedDate;
  qint64 size;
};
