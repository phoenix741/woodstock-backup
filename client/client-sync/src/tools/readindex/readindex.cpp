#include <QCoreApplication>
#include <QCommandLineParser>
#include <QDebug>
#include <QFileInfo>
#include <iostream>
#include <unordered_set>
#include <algorithm>

#include <manifest/manifest.h>
#include <protobuf/group_grpc.h>
#include <utils/sha256.h>
#include <manifest/indexmanifest.h>

#include "woodstock.pb.h"

using woodstock::FileManifestJournalEntry;
// https://gist.github.com/plasticbox/3708a6cdfbece8cd224487f9ca9794cd

void printIndex(IndexFileEntry entry, QString path)
{
    QString fullpath = path + "/" + entry.path();

    qInfo() << "Path: " << fullpath;
    qInfo() << "  Index: " << entry.index();
    qInfo() << "  IsJournal: " << entry.journal();

    for (auto file = entry.files().begin(); file != entry.files().end(); file++)
    {
        printIndex(file.value(), fullpath);
    }
}

void printManifestEntry(const woodstock::FileManifest &manifest, const QString &tab)
{
    qInfo() << tab << "Manifest: ";
    qInfo() << tab << "  Path: " << QString::fromStdString(manifest.path());
    qInfo() << tab << "  Stats: ";
    qInfo() << tab << "    OwnerId: " << manifest.stats().ownerid();
    qInfo() << tab << "    GroupId: " << manifest.stats().groupid();
    qInfo() << tab << "    Size: " << manifest.stats().size();
    qInfo() << tab << "    Last read: " << manifest.stats().lastread();
    qInfo() << tab << "    Last modified: " << manifest.stats().lastmodified();
    qInfo() << tab << "    Created: " << manifest.stats().created();
    qInfo() << tab << "    Mode: " << manifest.stats().mode();
    qInfo() << tab << "  SHA256: " << QByteArray::fromStdString(manifest.sha256()).toHex();
    qInfo() << tab << "  Chunks: ";
    for (auto chunk : manifest.chunks())
    {
        qInfo() << tab << "    " << QByteArray::fromStdString(chunk).toHex();
    }
}

void printJournalEntry(const woodstock::FileManifestJournalEntry &entry)
{
    qInfo() << "Type: " << entry.type();
    qInfo() << "  Path: " << QString::fromStdString(entry.path());
    printManifestEntry(entry.manifest(), "  ");
    qInfo();
}

int main(int argc, char *argv[])
{
    GOOGLE_PROTOBUF_VERIFY_VERSION;
    QCoreApplication app(argc, argv);
    QCoreApplication::setApplicationName("readindex");
    QCoreApplication::setApplicationVersion("1.0");

    QCommandLineParser parser;
    parser.setApplicationDescription("Read index and manifest file");
    parser.addHelpOption();
    parser.addVersionOption();

    parser.addOptions({
        {"manifest",
         QCoreApplication::translate("main", "Read a manifest file")},
        {"journal",
         QCoreApplication::translate("main", "Read a journal file")},
    });

    // Process the actual command line arguments given by the user
    parser.process(app);

    const QStringList files = parser.positionalArguments();

    bool isManifest = parser.isSet("manifest");
    bool isJournal = parser.isSet("journal");

    if (!isManifest && !isJournal)
    {
        for (auto file : files)
        {
            QFileInfo fileInfo(file);
            Manifest manifest(fileInfo.fileName().replace("." + fileInfo.completeSuffix(), ""), fileInfo.path());

            auto index = manifest.loadIndex();

            qInfo() << "Number of element in index: " << index->indexSize;

            printIndex(index->getEntry(""), "");
        }
    }

    if (isJournal)
    {
        for (auto file : files)
        {
            FileManifestJournalEntry entry;

            QFile fileStream(file);
            if (!fileStream.open(QFile::ReadOnly))
            {
                throw std::runtime_error("Can't open the file " + file.toStdString());
            }

            while (readDelimitedFrom(&fileStream, &entry))
            {
                printJournalEntry(entry);
                entry.clear_manifest();
            }

            fileStream.close(); // FIXME: close propre
        }
    }

    if (isManifest)
    {
        for (auto file : files)
        {
            woodstock::FileManifest m;

            QFile fileStream(file);
            if (!fileStream.open(QFile::ReadOnly))
            {
                throw std::runtime_error("Can't open the file " + file.toStdString());
            }
            while (readDelimitedFrom(&fileStream, &m))
            {
                printManifestEntry(m, "");
                m.clear_chunks();
                m.clear_acl();
                m.clear_xattr();
            }

            fileStream.close(); // FIXME: close propre
        }
    }

    return 0;
}
