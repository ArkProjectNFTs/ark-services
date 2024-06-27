/* eslint-disable @typescript-eslint/no-misused-promises */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
"use client";

import { useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { Button } from "~/components/ui/button";
import { Form, FormField, FormItem, FormLabel } from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Switch } from "~/components/ui/switch";
import { api } from "~/trpc/react";
import type { Contract } from "~/types";
import { Separator } from "../ui/separator";

const editCollectionFormSchema = z.object({
  name: z.string(),
  saveImages: z.boolean(),
  symbol: z.string(),
  isSpam: z.boolean(),
  isNSFW: z.boolean(),
  isVerified: z.boolean(),
  image: z.string().optional(),
});

type EditCollectionFormValues = z.infer<typeof editCollectionFormSchema>;

async function uploadFileToS3(file: File, presignedUrl: string) {
  try {
    const response = await fetch(presignedUrl, {
      method: "PUT",
      headers: {
        "Content-Type": file.type,
      },
      body: file,
    });

    if (!response.ok) {
      throw new Error("Failed to upload file!");
    }
  } catch (error) {
    console.error("Error uploading file:", error);
  }
}

export default function CollectionForm(props: { contract?: Contract }) {
  const [avatarMediaKey, setAvatarMediaKey] = useState<string>();
  const [file, setFile] = useState<File>();

  const form = useForm<EditCollectionFormValues>({
    resolver: zodResolver(editCollectionFormSchema),
    defaultValues: {
      saveImages: props.contract?.save_images ?? false,
      name: props.contract?.contract_name,
      symbol: props.contract?.contract_symbol,
      isSpam: props.contract?.is_spam ?? false,
      isNSFW: props.contract?.is_nsfw ?? false,
      isVerified: props.contract?.is_verified ?? false,
      image: props.contract?.contract_image,
    },
  });

  const { data: avatarPresignedUri } =
    api.media.generateMediaPresignedUri.useQuery(
      {
        mediaKey: avatarMediaKey ?? "",
      },
      {
        enabled: !!avatarMediaKey,
      },
    );

  const { mutateAsync, isLoading } = api.contract.updateContract.useMutation(
    {},
  );

  // async function onSubmit(data: EditCollectionFormValues) {
  //   if (props.contract?.contract_address) {
  //     await mutateAsync({
  //       contractAddress: props.contract.contract_address,
  //       image: data.image,
  //       isNSFW: data.isNSFW,
  //       isSpam: data.isSpam,
  //       isVerified: data.isVerified,
  //       name: data.name,
  //       saveImages: data.saveImages,
  //       symbol: data.symbol,
  //     });

  //     console.log("submitted!");
  //   } else {
  //     console.error("Contract address not found");
  //   }
  // }

  return (
    <Form {...form}>
      <form
        // onSubmit={async () => {
        //   if (file && avatarPresignedUri) {
        //     try {
        //       await uploadFileToS3(file, avatarPresignedUri);
        //       form.setValue(
        //         "image",
        //         `https://media.arkproject.dev/${avatarMediaKey}`,
        //       );
        //     } catch {}

        //     console.log("File uploaded to S3");
        //   }

        //   return form.handleSubmit(onSubmit);
        // }}
        className="space-y-4"
      >
        <FormItem>
          <FormLabel htmlFor="avatar-image">Image</FormLabel>
          {props.contract?.contract_image && (
            <div className="grid gap-2">
              <img
                src={props.contract?.contract_image}
                alt="Image Preview"
                width={200}
                className="mx-auto rounded-md"
              />
            </div>
          )}
          <Input
            onChange={(event) => {
              if (event.target?.files && event.target.files.length > 0) {
                const file = event.target.files[0];
                const fileExt = file?.name.split(".").pop();
                setAvatarMediaKey(
                  `contracts/${props.contract?.contract_address}/avatar.${fileExt}`,
                );
                setFile(file);
              }
            }}
            id="avatar-image"
            type="file"
          />
        </FormItem>

        <FormItem>
          <FormLabel>Type</FormLabel>
          <Input
            readOnly={true}
            contentEditable={false}
            value={props.contract?.contract_type}
            disabled={true}
          />
        </FormItem>

        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel htmlFor={field.name}>Name</FormLabel>
                  <Input {...field} />
                </FormItem>
              )}
            />
          </div>
          <div className="grid gap-2">
            <FormField
              control={form.control}
              name="symbol"
              render={({ field }) => (
                <FormItem>
                  <FormLabel htmlFor={field.name}>Symbol</FormLabel>
                  <Input {...field} />
                </FormItem>
              )}
            />
          </div>
        </div>

        <div>
          <h3 className="text-lg font-medium">Indexer Settings</h3>
          <p className="text-sm text-muted-foreground">
            These settings are specific to the indexer and do not affect the
            contract itself.
          </p>
        </div>
        <Separator />

        <FormField
          control={form.control}
          name="saveImages"
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between space-x-2">
                <Label htmlFor={field.name} className="flex flex-col space-y-1">
                  <span>☁️ Save Images</span>
                  <span className="font-normal leading-snug text-muted-foreground">
                    Save images to AWS S3 to cache them for faster loading
                  </span>
                </Label>
                <Switch
                  checked={field.value}
                  id={field.name}
                  onCheckedChange={(value) => {
                    form.setValue(field.name, value);
                  }}
                />
              </div>
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="isNSFW"
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between space-x-2">
                <Label htmlFor={field.name} className="flex flex-col space-y-1">
                  <span>🔞 Is NSFW</span>
                  <span className="font-normal leading-snug text-muted-foreground">
                    Mark this collection as NSFW
                  </span>
                </Label>
                <Switch
                  checked={field.value}
                  id={field.name}
                  onCheckedChange={(value) => {
                    form.setValue(field.name, value);
                  }}
                />
              </div>
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="isSpam"
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between space-x-2">
                <Label htmlFor={field.name} className="flex flex-col space-y-1">
                  <span>⛔ Is Spam</span>
                  <span className="font-normal leading-snug text-muted-foreground">
                    Mark this collection as spam
                  </span>
                </Label>
                <Switch
                  checked={field.value}
                  id={field.name}
                  onCheckedChange={(value) => {
                    form.setValue(field.name, value);
                  }}
                />
              </div>
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="isVerified"
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between space-x-2">
                <Label htmlFor={field.name} className="flex flex-col space-y-1">
                  <span>✅ Is Verified</span>
                  <span className="font-normal leading-snug text-muted-foreground">
                    Mark this collection as verified
                  </span>
                </Label>
                <Switch
                  checked={field.value}
                  id={field.name}
                  onCheckedChange={(value) => {
                    form.setValue(field.name, value);
                  }}
                />
              </div>
            </FormItem>
          )}
        />

        <Button
          type="button"
          onClick={async () => {
            if (props.contract?.contract_address) {
              if (file && avatarPresignedUri) {
                await uploadFileToS3(file, avatarPresignedUri);
              }

              const data = form.getValues();
              await mutateAsync({
                contractAddress: props.contract.contract_address,
                image: avatarMediaKey
                  ? `https://media.arkproject.dev/${avatarMediaKey}`
                  : undefined,
                isNSFW: data.isNSFW,
                isSpam: data.isSpam,
                isVerified: data.isVerified,
                name: data.name,
                saveImages: data.saveImages,
                symbol: data.symbol,
              });
            }
          }}
          className="w-full"
        >
          Save
        </Button>
      </form>
    </Form>
  );
}
