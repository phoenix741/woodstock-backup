#include "woodstockserver.h"

#include "request-server/request-server-impl.h"
#include <rxcpp/rx.hpp>
#include <rxcpp/rx-observable.hpp>
#include <manifest/manifest.h>
#include <manifest/indexmanifest.h>
#include <pool/pool.h>
#include <pool/pool-chunk-refcnt.h>
#include <utils/sha256.h>
#include <sys/stat.h>

#include <QTime>
#include <iomanip>
#include <QLocale>
#include <QDebug>

WoodstockServer::WoodstockServer()
{
}

WoodstockServer::~WoodstockServer()
{
}

Common::Configuration WoodstockServer::getConfiguration(const QString &host) const
{
  Common::Configuration configuration;

  Common::Share share("/home");
  // share.includes().append("*backupPhoto.7z");

  share.excludes().append("phoenix/tensorflow");
  share.excludes().append("phoenix/tmp");
  share.excludes().append("phoenix/.composer");
  share.excludes().append("*node_modules");
  share.excludes().append("*mongodb/db");
  share.excludes().append("phoenix/.ccache");
  share.excludes().append("*mongodb/dump");
  share.excludes().append("phoenix/usr/android-sdk");
  share.excludes().append("phoenix/.cache");
  share.excludes().append("phoenix/.CloudStation");
  share.excludes().append("phoenix/.android");
  share.excludes().append("phoenix/.AndroidStudio*");
  share.excludes().append("phoenix/usr/android-studio");
  share.excludes().append("*.vmdk");
  share.excludes().append("phoenix/.nvm");
  share.excludes().append("*.vdi");
  share.excludes().append("phoenix/.local/share/Trash");
  share.excludes().append("phoenix/VirtualBox VMs");
  share.excludes().append("*mongodb/configdb");
  share.excludes().append("phoenix/.thumbnails");
  share.excludes().append("phoenix/.VirtualBox");
  share.excludes().append("phoenix/.vagrant.d");
  share.excludes().append("phoenix/vagrant");
  share.excludes().append("phoenix/.npm");
  share.excludes().append("phoenix/Pictures");
  share.excludes().append("phoenix/Documents synhronisés");
  share.excludes().append("phoenix/dwhelper");
  share.excludes().append("phoenix/snap");
  share.excludes().append("phoenix/.local/share/flatpak");
  share.excludes().append("phoenix/usr/AndroidSdk");
  share.excludes().append("public/kg/gallery");
  share.excludes().append("*vcpkg");

  Common::Share shareEtc("/etc");

  configuration.operations().tasks().push_back(Common::Task("/usr/bin/which which"));
  configuration.operations().tasks().push_back(Common::Task({share, shareEtc}));
  configuration.operations().finalisedTasks().push_back(Common::Task("/usr/bin/which which"));

  return configuration;
}

void WoodstockServer::start()
{
  nbFileRead = 0;
  totalSize = 0;
  transferredSize = 0;
  compressedPoolSize = 0;
  m_timer.start();
  m_lastElapsed = 0;
  printStats();

  QString hostToBackup("pc-ulrich.eden.lan");
  qint32 lastBackupId = -1;
  qint32 currentBackupId = 0;

  std::unique_ptr<QLockFile> lockFile;
  BackupPool pool("/home/phoenix/tmp/woodstock/pool");

  try
  {
    // Get the configuration for the client
    qDebug() << QString("Get the configuration of %1").arg(hostToBackup);
    auto configuration = getConfiguration(hostToBackup);

    // Create the connection with the client
    qDebug() << QString("Connect to %1").arg(hostToBackup);
    auto server = getRequestServer(hostToBackup);

    // Ask the client to prepare to backup
    qDebug() << QString("Prepare the backup of %1 number %2").arg(hostToBackup).arg(currentBackupId);
    auto prepareResult = server->prepareBackup(configuration, lastBackupId, currentBackupId);

    // Refresh the cache
    qDebug() << "Refresh the cache (TODO)";
    /*
    if (prepareResult.needRefreshCache && lastBackupId)
    {
      std::unique_ptr<Manifest> previousManifest(new Manifest(QString("backup.%1").arg(lastBackupId), "/home/phoenix/tmp/woodstock/hosts/" + hostToBackup));
      server->refreshCache(previousManifest->getManifestWrapper());
    }
    */

    // Launch the backup
    qDebug() << "Create the pool and read the manifest";
    std::unique_ptr<Manifest> manifest(new Manifest(QString("backup.%1").arg(currentBackupId), "/home/phoenix/tmp/woodstock/hosts/" + hostToBackup));
    auto index = manifest->loadIndex();
    qDebug() << "Size of index " << index->indexSize;

    lockFile = std::unique_ptr<QLockFile>(new QLockFile(manifest->lockPath()));
    // FIXME: Manifest copy from previous version

    qDebug() << "Lock " << manifest->lockPath();
    if (!lockFile->lock())
    {
      throw std::runtime_error("Can't lock the backup"); // TODO: Exception
    }

    qDebug() << "Launch the backup " << currentBackupId;
    bool finishFlag = false;

    auto journalEntries = rxcpp::observable<>::create<const Common::JournalEntry>([&server, currentBackupId](rxcpp::subscriber<const Common::JournalEntry> s) {
      server->launchBackup(currentBackupId, [&s](const Common::JournalEntry &e) {
        s.on_next(e);
      });
      s.on_completed();
    });

    journalEntries
        .flat_map(
            [this, &finishFlag, &index, &manifest, &pool, &server](const Common::JournalEntry &e) {
              return rxcpp::observable<>::create<const Common::JournalEntry>([this, &e, &finishFlag, &index, &manifest, &pool, &server](rxcpp::subscriber<const Common::JournalEntry> s) {
                auto entry = e;

                if (entry.type() == Common::JournalEntryType::CLOSE)
                {
                  finishFlag = true;
                  s.on_next(entry);
                  return;
                }

                auto path = entry.manifest().isValid() ? entry.manifest().path() : entry.path();
                // qDebug() << "  --> Receive the file " << path;
                try
                {
                  // Transfert missing chunk
                  nbFileRead++;
                  if (entry.type() != Common::REMOVE)
                  {
                    auto indexEntry = index->getEntry(path);
                    auto originalFile = manifest->getManifest(indexEntry);
                    // qDebug() << QString("    --> Original manifest %1").arg(!!originalFile ? " exist" : " not exist");

                    if ((entry.manifest().stats().mode() & S_IFMT) == S_IFREG)
                    {
                      auto length = (entry.manifest().stats().size() / CHUNK_SIZE) + 1;
                      for (auto i = 0; i < length; i++)
                      {
                        auto originalFileChunk = (originalFile.isValid() && (i < originalFile.chunks().size())) ? originalFile.chunks().at(i) : QByteArray();
                        auto fileChunk = (i < entry.manifest().chunks().size()) ? entry.manifest().chunks().at(i) : QByteArray();
                        auto fileChunkHex = fileChunk.toHex();

                        if (fileChunk.isEmpty() || (fileChunk != originalFileChunk))
                        {
                          // qDebug() << "    --> Should transfert " << fileChunkHex;

                          // Transfert fileChunk
                          if (pool.isChunkExists(fileChunk))
                          {
                            // qDebug() << "    --> The chunk already exists " << fileChunkHex;
                            continue;
                          }

                          // qDebug() << "    --> Transfert chunk " << fileChunkHex;
                          auto wrapper = pool.getChunk(fileChunk);
                          auto size = server->getChunk(path, i * CHUNK_SIZE, CHUNK_SIZE, wrapper.get());

                          transferredSize += wrapper->getDevice()->size();
                          compressedPoolSize += wrapper->getFile()->size();
                          auto sha256 = wrapper->checkAndClose();
                          if (sha256 != fileChunk || fileChunk.isEmpty())
                          {
                            // Update the sha256 in the manifest
                            if (i < entry.manifest().chunks().size())
                            {
                              entry.manifest().chunks()[i] = sha256;
                            }
                            else
                            {
                              entry.manifest().chunks().append(sha256);
                            }
                          }

                          printStats();

                          // qDebug() << "    --> End";
                        }
                        totalSize += entry.manifest().stats().size();
                      }
                    }
                  }
                  // qDebug() << "--> End";
                  printStats();

                  // FIXME: Push the file list to client if modified ?
                  // FIXME: Or when reading chunk in the client side ?

                  // Store the file
                  if (entry.type() == Common::REMOVE)
                  {
                    manifest->removePath(entry.path());
                  }
                  else
                  {
                    manifest->addManifest(entry.manifest(), entry.type() == Common::ADD);
                  }

                  s.on_next(entry);
                }
                catch (std::runtime_error &e)
                {
                  // s.on_error(std::exception_ptr());
                  qCritical() << QString("Can't backup the file %1").arg(path);
                }
                s.on_completed();
              });
            })
        .as_blocking()
        .subscribe(
            [](Common::JournalEntry e) {},
            [](std::exception_ptr ep) {
              try
              {
                std::rethrow_exception(ep);
              }
              catch (const std::exception &ex)
              {
                printf("OnError: %s\n", ex.what());
              }
            },
            []() { printf("OnCompleted\n"); });

    // Backup is finish, close the backup
    if (finishFlag)
    {
      auto refcnt = PoolChunkRefCnt::get("/home/phoenix/tmp/woodstock/pool");
      manifest->compact([&refcnt](const Common::FileManifest &manifest) {
        for (auto chunk : manifest.chunks())
        {
          refcnt->incr(chunk);
        }
      });
    }
    else
    {
      qWarning() << "Backup not finished";
    }
  }
  catch (std::runtime_error &error)
  {
    qCritical() << QString("Can't backup the host %1: %2").arg(hostToBackup).arg(error.what());
  }
  lockFile->unlock();
}

void WoodstockServer::printStats()
{
  if (m_timer.elapsed() - m_lastElapsed > 5000)
  {
    std::cout << "\33[2K\r[" << QTime::fromMSecsSinceStartOfDay(m_timer.elapsed()).toString().toStdString() << "]" << std::setw(9) << std::setfill(' ') << nbFileRead << " files read / " << QLocale::c().formattedDataSize(totalSize).toStdString() << " total / " << QLocale::c().formattedDataSize(transferredSize).toStdString() << " transferred / " << QLocale::c().formattedDataSize(compressedPoolSize).toStdString() << " compressed" << std::flush;
    m_lastElapsed = m_timer.elapsed();
  }
}